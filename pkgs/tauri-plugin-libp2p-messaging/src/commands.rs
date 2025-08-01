use crate::{AppState, Error, Libp2pCommand};
use tauri::{async_runtime::channel, command, AppHandle, Runtime, State, Window};

// 订阅指定主题。
///
/// # 参数
/// - `topic`: 要订阅的主题名称。
///
/// # 返回值
/// - `Ok(())`: 订阅成功。
/// - `Err(Error)`: 订阅失败，返回错误信息。
#[command]
pub async fn subscribe_topic<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, AppState>,
  topic: String,
) -> Result<(), Error> {
  // Check if sender is closed
  if state.command_sender.is_closed() {
    return Err(Error::ChannelClosed);
  }

  state
    .command_sender
    .send(Libp2pCommand::Subscribe(topic))
    .await
    .map_err(|e| Error::ChannelSend(e.to_string()))?;
  Ok(())
}

/// 取消订阅指定主题。
///
/// # 参数
/// - `topic`: 要取消订阅的主题名称。
///
/// # 返回值
/// - `Ok(())`: 取消订阅成功。
/// - `Err(Error)`: 取消订阅失败，返回错误信息。
#[command]
pub async fn unsubscribe_topic<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, AppState>,
  topic: String,
) -> Result<(), Error> {
  state
    .command_sender
    .send(Libp2pCommand::Unsubscribe(topic))
    .await?;
  Ok(())
}

/// 向指定主题发送消息。
///
/// # 参数
/// - `topic`: 目标主题名称。
/// - `message`: 要发送的消息内容。
///
/// # 返回值
/// - `Ok(())`: 消息发送成功。
/// - `Err(Error)`: 消息发送失败，返回错误信息。
#[command]
pub async fn send_message<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, AppState>,
  topic: String,
  message: String,
) -> Result<(), Error> {
  state
    .command_sender
    .send(Libp2pCommand::SendMessage(topic, message.into_bytes()))
    .await?;
  Ok(())
}

/// 获取当前连接的节点及其支持的主题列表。
///
/// # 返回值
/// - `Ok(Vec<(String, Vec<String>)>)`: 成功返回节点及其主题列表。
/// - `Err(Error)`: 获取失败，返回错误信息。
#[command]
pub async fn get_peers<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, AppState>,
) -> Result<Vec<(String, Vec<String>)>, Error> {
  // 创建一个 oneshot 通道
  let (sender, mut receiver) = channel(1);

  // 发送 GetPeers 命令，携带 sender
  state
    .command_sender
    .send(Libp2pCommand::GetPeers(sender))
    .await
    .map_err(|e| Error::ChannelSend(e.to_string()))?;

  // 等待后台线程返回结果
  receiver
    .recv()
    .await
    .ok_or_else(|| Error::ChannelReceive("Channel closed".to_string()))
}
