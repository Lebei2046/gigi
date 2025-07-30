use crate::{Error, Libp2pMessaging};
use tauri::{command, State, Wry};

#[command]
pub async fn subscribe_topic(
  topic: String,
  state: State<'_, tokio::sync::Mutex<Libp2pMessaging<Wry>>>,
) -> Result<(), Error> {
  let mut messaging = state.inner().lock().await;
  messaging.subscribe(&topic)
}

#[command]
pub async fn unsubscribe_topic(
  topic: String,
  state: State<'_, tokio::sync::Mutex<Libp2pMessaging<Wry>>>,
) -> Result<(), Error> {
  let mut messaging = state.inner().lock().await;
  messaging.unsubscribe(&topic)
}

#[command]
pub async fn send_message(
  topic: String,
  message: String,
  state: State<'_, tokio::sync::Mutex<Libp2pMessaging<Wry>>>,
) -> Result<(), Error> {
  let mut messaging = state.inner().lock().await;
  messaging.publish(&topic, message.as_bytes())
}

#[command]
pub async fn get_peers(
  state: State<'_, tokio::sync::Mutex<Libp2pMessaging<Wry>>>,
) -> Result<Vec<(String, Vec<String>)>, Error> {
  let messaging = state.inner().lock().await;
  Ok(messaging.get_peers())
}
