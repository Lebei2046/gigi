use crate::{AppState, Error, Libp2pCommand};
use tauri::{command, AppHandle, Runtime, State, Window};

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

#[command]
pub async fn get_peers<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, AppState>,
) -> Result<Vec<(String, Vec<String>)>, Error> {
  // 创建一个 oneshot 通道
  let (sender, receiver) = tokio::sync::oneshot::channel();

  // 发送 GetPeers 命令，携带 sender
  state
    .command_sender
    .send(Libp2pCommand::GetPeers(sender))
    .await
    .map_err(|e| Error::ChannelSend(e.to_string()))?;

  // 等待后台线程返回结果
  receiver
    .await
    .map_err(|e| Error::ChannelReceive(e.to_string()))
}
