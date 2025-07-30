use libp2p::{
  futures::StreamExt as _,
  gossipsub::{self, IdentTopic, MessageAuthenticity},
  mdns,
  swarm::{NetworkBehaviour, SwarmEvent},
  PeerId, Swarm,
};
use std::collections::HashSet;
use tauri::{
  plugin::{Builder, TauriPlugin},
  Emitter as _, Manager, Runtime,
};

mod commands;
mod error;
mod models;

pub use error::Error;
pub use models::{MessageReceivedEvent, PeerDiscoveredEvent};

#[derive(NetworkBehaviour)]
struct Libp2pMessagingBehaviour {
  gossipsub: gossipsub::Behaviour,
  mdns: mdns::tokio::Behaviour,
}

pub struct Libp2pMessaging<R: Runtime> {
  swarm: Swarm<Libp2pMessagingBehaviour>,
  app_handle: tauri::AppHandle<R>,
  subscribed_topics: HashSet<String>,
}

impl<R: Runtime> Libp2pMessaging<R> {
  pub fn new(app_handle: tauri::AppHandle<R>) -> Result<Self, Error> {
    let gossipsub_config = gossipsub::ConfigBuilder::default()
      .max_transmit_size(262144)
      .build()
      .map_err(|e| Error::GossipsubConfigError(e.to_string()))?;

    let gossipsub = gossipsub::Behaviour::new(
      MessageAuthenticity::Signed(libp2p::identity::Keypair::generate_ed25519()),
      gossipsub_config,
    )
    .map_err(|e| Error::GossipsubError(e.to_string()))?;

    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), PeerId::random())
      .map_err(|e| Error::MdnsError(e.to_string()))?;

    let behaviour = Libp2pMessagingBehaviour { gossipsub, mdns };

    let swarm = libp2p::SwarmBuilder::with_new_identity()
      .with_tokio()
      .with_tcp(
        libp2p::tcp::Config::default(),
        libp2p::noise::Config::new,
        libp2p::yamux::Config::default,
      )
      .map_err(|e| Error::SwarmError(e.to_string()))?
      .with_behaviour(|_key| behaviour)
      .map_err(|e| Error::BehaviourError(e.to_string()))?
      .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
      .build();

    Ok(Self {
      swarm,
      app_handle,
      subscribed_topics: HashSet::new(),
    })
  }

  pub fn subscribe(&mut self, topic: &str) -> Result<(), Error> {
    let topic = IdentTopic::new(topic);
    self
      .swarm
      .behaviour_mut()
      .gossipsub
      .subscribe(&topic)
      .map_err(|e| Error::SubscriptionError(e.to_string()))?;
    self.subscribed_topics.insert(topic.to_string());
    Ok(())
  }

  pub fn unsubscribe(&mut self, topic: &str) -> Result<(), Error> {
    let topic = IdentTopic::new(topic);
    self
      .swarm
      .behaviour_mut()
      .gossipsub
      .unsubscribe(&topic)
      .map_err(|e| Error::SubscriptionError(e.to_string()))?;
    self.subscribed_topics.remove(topic.to_string().as_str());
    Ok(())
  }

  pub fn publish(&mut self, topic: &str, message: &[u8]) -> Result<(), Error> {
    let topic = IdentTopic::new(topic);
    self
      .swarm
      .behaviour_mut()
      .gossipsub
      .publish(topic, message)
      .map_err(|e| Error::PublishError(e.to_string()))?;
    Ok(())
  }

  pub fn get_peers(&self) -> Vec<(String, Vec<String>)> {
    self
      .swarm
      .behaviour()
      .mdns
      .discovered_nodes()
      .map(|peer_id| {
        // Since we can't get addresses per peer from Behaviour after the fact,
        // return empty vec or manage a local map if needed.
        (peer_id.to_string(), vec![])
      })
      .collect()
  }

  pub async fn run(&mut self) {
    self
      .swarm
      .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
      .unwrap();

    loop {
      tokio::select! {
          event = self.swarm.select_next_some() => match event {
              SwarmEvent::Behaviour(Libp2pMessagingBehaviourEvent::Gossipsub(
                  gossipsub::Event::Message {
                      propagation_source: peer_id,
                      message_id: _,
                      message,
                  }
              )) => {
                  let _ = self.app_handle.emit("message-received", MessageReceivedEvent {
                      topic: message.topic.into_string(),
                      data: String::from_utf8_lossy(&message.data).to_string(),
                      sender: peer_id.to_string(),
                  });
              }
              SwarmEvent::Behaviour(Libp2pMessagingBehaviourEvent::Mdns(
                  mdns::Event::Discovered(peers)
              )) => {
                  for (peer_id, addr) in peers {
                      let _ = self.app_handle.emit("peer-discovered", PeerDiscoveredEvent {
                          id: peer_id.to_string(),
                          addresses: vec![addr.to_string()],
                      });
                  }
              }
              SwarmEvent::NewListenAddr { address, .. } => {
                  println!("Listening on {:?}", address);
              }
              _ => {}
          }
      }
    }
  }
}

// Add this at the end of src/lib.rs
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("libp2p-messaging")
    .setup(|app, _api| {
      let app_handle = app.clone();

      let messaging = Libp2pMessaging::new(app_handle.clone())
        .map_err(|e| format!("Failed to initialize libp2p messaging: {}", e))?;

      app.manage(tokio::sync::Mutex::new(messaging));

      tauri::async_runtime::spawn(async move {
        let state: tauri::State<tokio::sync::Mutex<Libp2pMessaging<R>>> = app_handle.state();
        let mut messaging = state.inner().lock().await;
        messaging.run().await;
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::subscribe_topic,
      commands::unsubscribe_topic,
      commands::send_message,
      commands::get_peers
    ])
    .build()
}
