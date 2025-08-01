use libp2p::{
  futures::StreamExt as _,
  gossipsub::{self, IdentTopic, MessageAuthenticity},
  mdns,
  swarm::{NetworkBehaviour, SwarmEvent},
  PeerId, Swarm,
};
use std::collections::HashSet;
use tauri::{async_runtime::Receiver, Emitter as _, Runtime};

use crate::{
  error::Error,
  models::{MessageReceivedEvent, PeerDiscoveredEvent},
  Libp2pCommand,
};

/// 定义 Libp2p 消息传递的行为，包括 Gossipsub 和 mDNS 功能。
#[derive(NetworkBehaviour)]
struct Libp2pMessagingBehaviour {
  gossipsub: gossipsub::Behaviour,
  mdns: mdns::tokio::Behaviour,
}

/// 提供 Libp2p 消息传递功能的主结构体。
///
/// # 泛型参数
/// - `R`: 运行时类型，必须实现 `tauri::Runtime` trait。
pub struct Libp2pMessaging<R: Runtime> {
  swarm: Swarm<Libp2pMessagingBehaviour>,
  app_handle: tauri::AppHandle<R>,
  subscribed_topics: HashSet<String>,
  command_receiver: Receiver<Libp2pCommand>,
}

impl<R: Runtime> Libp2pMessaging<R> {
  /// 创建一个新的 `Libp2pMessaging` 实例。
  ///
  /// # 参数
  /// - `app_handle`: Tauri 应用句柄。
  /// - `receiver`: 用于接收 Libp2p 命令的通道接收端。
  ///
  /// # 返回
  /// - `Result<Self, Error>`: 成功时返回实例，失败时返回错误。
  pub fn new(
    app_handle: tauri::AppHandle<R>,
    receiver: Receiver<Libp2pCommand>,
  ) -> Result<Self, Error> {
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
      command_receiver: receiver,
    })
  }

  /// 订阅指定主题。
  ///
  /// # 参数
  /// - `topic`: 要订阅的主题名称。
  ///
  /// # 返回
  /// - `Result<(), Error>`: 成功时返回 `Ok(())`，失败时返回错误。
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

  /// 取消订阅指定主题。
  ///
  /// # 参数
  /// - `topic`: 要取消订阅的主题名称。
  ///
  /// # 返回
  /// - `Result<(), Error>`: 成功时返回 `Ok(())`，失败时返回错误。
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

  /// 向指定主题发布消息。
  ///
  /// # 参数
  /// - `topic`: 目标主题名称。
  /// - `message`: 要发布的消息内容。
  ///
  /// # 返回
  /// - `Result<(), Error>`: 成功时返回 `Ok(())`，失败时返回错误。
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

  /// 获取当前发现的节点列表。
  ///
  /// # 返回
  /// - `Vec<(String, Vec<String>)>`: 节点 ID 及其地址列表。
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

  /// 启动 Libp2p 消息传递服务的主循环。
  ///
  /// 该方法会持续监听事件并处理命令。
  pub async fn run(&mut self) {
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
                let _ = sender.send(peers); // 发送结果
            },
          }
        },
        event = self.swarm.select_next_some() => match event {
          SwarmEvent::Behaviour(Libp2pMessagingBehaviourEvent::Gossipsub(
            gossipsub::Event::Message {
              propagation_source: peer_id,
              message_id: _,
              message,
            }
          )) => {
            let _ = self.app_handle.emit("plugin:libp2p-messageing:message-received", MessageReceivedEvent {
              topic: message.topic.into_string(),
              data: String::from_utf8_lossy(&message.data).to_string(),
              sender: peer_id.to_string(),
            });
          }
          SwarmEvent::Behaviour(Libp2pMessagingBehaviourEvent::Mdns(
            mdns::Event::Discovered(peers)
          )) => {
            for (peer_id, addr) in peers {
              let _ = self.app_handle.emit("plugin:libp2p-messageing:peer-discovered", PeerDiscoveredEvent {
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
