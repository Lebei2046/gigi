use libp2p::{
  futures::StreamExt as _,
  gossipsub::{self, IdentTopic},
  mdns, noise,
  swarm::{NetworkBehaviour, SwarmEvent},
  tcp, yamux, Swarm,
};
use std::{
  collections::HashSet,
  hash::{DefaultHasher, Hash, Hasher},
  io,
  time::Duration,
};
use tauri::{async_runtime::Receiver, Emitter as _, Runtime};

use crate::{
  error::Error,
  models::{MessageReceivedEvent, PeerDiscoveredEvent},
  Libp2pCommand,
};

/// Defines the behavior of Libp2p messaging, including Gossipsub and mDNS functionality.
#[derive(NetworkBehaviour)]
struct Libp2pMessagingBehaviour {
  gossipsub: gossipsub::Behaviour,
  mdns: mdns::tokio::Behaviour,
}

/// Main structure providing Libp2p messaging functionality.
///
/// # Generic parameters
/// - `R`: Runtime type, must implement `tauri::Runtime` trait.
pub struct Libp2pMessaging<R: Runtime> {
  swarm: Swarm<Libp2pMessagingBehaviour>,
  app_handle: tauri::AppHandle<R>,
  subscribed_topics: HashSet<String>,
  command_receiver: Receiver<Libp2pCommand>,
}

impl<R: Runtime> Libp2pMessaging<R> {
  /// Create a new `Libp2pMessaging` instance.
  ///
  /// # Parameters
  /// - `app_handle`: Tauri application handle.
  /// - `receiver`: Channel receiver end for receiving Libp2p commands.
  ///
  /// # Returns
  /// - `Result<Self, Error>`: Returns instance on success, returns error on failure.
  pub fn new(
    app_handle: tauri::AppHandle<R>,
    receiver: Receiver<Libp2pCommand>,
  ) -> Result<Self, Error> {
    let swarm = libp2p::SwarmBuilder::with_new_identity()
      .with_tokio()
      .with_tcp(
        tcp::Config::default(),
        noise::Config::new,
        yamux::Config::default,
      )?
      .with_quic()
      .with_behaviour(|key| {
        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &gossipsub::Message| {
          let mut s = DefaultHasher::new();
          message.data.hash(&mut s);
          gossipsub::MessageId::from(s.finish().to_string())
        };

        // Set a custom gossipsub configuration
        let gossipsub_config = gossipsub::ConfigBuilder::default()
          .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
          .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message
          // signing)
          .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
          .build()
          .map_err(io::Error::other)?; // Temporary hack because `build` does not return a proper `std::error::Error`.

        // build a gossipsub network behaviour
        let gossipsub = gossipsub::Behaviour::new(
          gossipsub::MessageAuthenticity::Signed(key.clone()),
          gossipsub_config,
        )?;

        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
        Ok(Libp2pMessagingBehaviour { gossipsub, mdns })
      })?
      .build();

    Ok(Self {
      swarm,
      app_handle,
      subscribed_topics: HashSet::new(),
      command_receiver: receiver,
    })
  }

  /// Subscribe to a specified topic.
  ///
  /// # Parameters
  /// - `topic`: The name of the topic to subscribe to.
  ///
  /// # Returns
  /// - `Result<(), Error>`: Returns `Ok(())` on success, returns error on failure.
  pub fn subscribe(&mut self, topic: &str) -> Result<(), Error> {
    let topic = IdentTopic::new(topic);
    self
      .swarm
      .behaviour_mut()
      .gossipsub
      .subscribe(&topic)
      .map_err(|e| Error::SubscriptionError(e.to_string()))?;
    self.subscribed_topics.insert(topic.to_string());
    println!("Subscribed to topic: {}", topic);
    Ok(())
  }

  /// Unsubscribe from a specified topic.
  ///
  /// # Parameters
  /// - `topic`: The name of the topic to unsubscribe from.
  ///
  /// # Returns
  /// - `Result<(), Error>`: Returns `Ok(())` on success, returns error on failure.
  pub fn unsubscribe(&mut self, topic: &str) -> Result<(), Error> {
    let topic = IdentTopic::new(topic);
    self.swarm.behaviour_mut().gossipsub.unsubscribe(&topic);
    self.subscribed_topics.remove(topic.to_string().as_str());
    Ok(())
  }

  /// Publish message to a specified topic.
  ///
  /// # Parameters
  /// - `topic`: Target topic name.
  /// - `message`: The message content to be published.
  ///
  /// # Returns
  /// - `Result<(), Error>`: Returns `Ok(())` on success, returns error on failure.
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

  /// Get the list of currently discovered nodes.
  ///
  /// # Returns
  /// - `Vec<(String, Vec<String>)>`: Node IDs and their address lists.
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

  /// Start the main loop of Libp2p messaging service.
  ///
  /// This method will continuously listen for events and handle commands.
  pub async fn run(&mut self) {
    self
      .swarm
      .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
      .unwrap();
    self
      .swarm
      .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
      .unwrap();

    println!("Libp2p messaging started, waiting for events...");

    loop {
      tokio::select! {
        Some(cmd) = self.command_receiver.recv() => {
          match cmd {
            Libp2pCommand::Subscribe(topic) => {
              if let Err(e) = self.subscribe(&topic) {
                eprintln!("Failed to subscribe: {}", e);
              }
            },
            Libp2pCommand::Unsubscribe(topic) => {
              if let Err(e) = self.unsubscribe(&topic) {
                eprintln!("Failed to unsubscribe: {}", e);
              }
            },
            Libp2pCommand::SendMessage(topic, message) => {
              if let Err(e) = self.publish(&topic, &message) {
                eprintln!("Failed to send message: {}", e);
              }
            },
            Libp2pCommand::GetPeers(sender) => {
                let peers = self.get_peers();
                let _ = sender.send(peers); // Send result
            },
          }
        },
        event = self.swarm.select_next_some() => match event {
          SwarmEvent::Behaviour(Libp2pMessagingBehaviourEvent::Gossipsub(
            gossipsub::Event::Message {
              propagation_source: peer_id,
              message_id: id,
              message,
            }
          )) => {
            println!("Got message: '{}' with id: {id} from peer: {peer_id}",
              String::from_utf8_lossy(&message.data),
            );

            let _ = self.app_handle.emit("gigi-messaging:message-received", MessageReceivedEvent {
              topic: message.topic.into_string(),
              data: String::from_utf8_lossy(&message.data).to_string(),
              sender: peer_id.to_string(),
            });
          }
          SwarmEvent::Behaviour(Libp2pMessagingBehaviourEvent::Mdns(
            mdns::Event::Discovered(peers)
          )) => {
            for (peer_id, addr) in peers {
              println!("mDNS discovered a new peer: {peer_id}");
              self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);

              let _ = self.app_handle.emit("gigi-messaging:peer-discovered", PeerDiscoveredEvent {
                id: peer_id.to_string(),
                addresses: vec![addr.to_string()],
              });
            }
          }
          SwarmEvent::Behaviour(Libp2pMessagingBehaviourEvent::Mdns(
            mdns::Event::Expired(list)
          )) => {
            for (peer_id, _multiaddr) in list {
              println!("mDNS discover peer has expired: {peer_id}");
              self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
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
