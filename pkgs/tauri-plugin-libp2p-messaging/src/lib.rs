/// 提供与 libp2p 消息传递相关的功能模块。
///
/// 该模块包含以下主要组件：
/// - `Libp2pMessaging`: 用于管理 libp2p 消息传递的核心功能。
/// - `Error`: 定义 libp2p 操作中可能出现的错误类型。
/// - `MessageReceivedEvent` 和 `PeerDiscoveredEvent`: 定义消息接收和对等节点发现的事件模型。
///
/// 模块还提供了以下功能：
/// - 订阅和取消订阅主题。
/// - 发送消息到指定主题。
/// - 获取当前连接的对等节点列表。
///
/// 使用 `init` 函数初始化插件并将其集成到 Tauri 应用中。
use tauri::{
  async_runtime::{channel, spawn, Sender},
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

mod commands;
mod desktop;
mod error;
mod models;

pub use desktop::Libp2pMessaging;
pub use error::Error;
pub use models::{MessageReceivedEvent, PeerDiscoveredEvent};

/// 表示 libp2p 命令的枚举类型。
///
/// 包含以下命令：
/// - `Subscribe`: 订阅指定主题。
/// - `Unsubscribe`: 取消订阅指定主题。
/// - `SendMessage`: 发送消息到指定主题。
/// - `GetPeers`: 获取当前连接的对等节点列表。
pub enum Libp2pCommand {
  Subscribe(String),
  Unsubscribe(String),
  SendMessage(String, Vec<u8>),
  GetPeers(Sender<Vec<(String, Vec<String>)>>),
}

/// 应用状态管理结构体。
///
/// 包含一个命令发送器 (`command_sender`)，用于向 libp2p 任务发送命令。
pub struct AppState {
  command_sender: Sender<Libp2pCommand>,
}

// 初始化 libp2p 消息传递插件。
///
/// 该函数会：
/// 1. 创建一个命令通道用于与 libp2p 任务通信。
/// 2. 将应用状态 (`AppState`) 注册到 Tauri 应用中。
/// 3. 启动 libp2p 任务以处理消息传递逻辑。
///
/// 返回一个配置好的 `TauriPlugin` 实例，可以集成到 Tauri 应用中。
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("libp2p-messaging")
    .setup(|app, _api| {
      let (command_sender, command_receiver) = channel(100);
      app.manage(AppState { command_sender });

      let app_handle = app.clone();
      // Spawn the libp2p task
      spawn(async move {
        match Libp2pMessaging::new(app_handle, command_receiver) {
          Ok(mut messaging) => {
            messaging.run().await;
          }
          Err(e) => {
            eprintln!("Failed to initialize Libp2pMessaging: {:?}", e);
          }
        }
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
