use hex::decode;
use libp2p::identity;
use tauri::{AppHandle, Runtime, State, Window, async_runtime::channel, command};

use crate::{AppState, Error, Libp2pCommand};

#[command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
pub fn get_peer_id(priv_key: &str) -> String {
    let bytes = decode(priv_key).unwrap().to_vec();
    let id_keys = identity::Keypair::ed25519_from_bytes(bytes).unwrap();
    let peer_id = id_keys.public().to_peer_id();

    peer_id.to_string()
}

// Subscribe to a specified topic.
///
/// # Parameters
/// - `topic`: The name of the topic to subscribe to.
///
/// # Return value
/// - `Ok(())`: Subscription successful.
/// - `Err(Error)`: Subscription failed, returns error information.
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

/// Unsubscribe from a specified topic.
///
/// # Parameters
/// - `topic`: The name of the topic to unsubscribe from.
///
/// # Return value
/// - `Ok(())`: Unsubscription successful.
/// - `Err(Error)`: Unsubscription failed, returns error information.
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

/// Send message to a specified topic.
///
/// # Parameters
/// - `topic`: Target topic name.
/// - `message`: The message content to be sent.
///
/// # Return value
/// - `Ok(())`: Message sent successfully.
/// - `Err(Error)`: Message sending failed, returns error information.
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

/// Get currently connected nodes and their supported topic lists.
///
/// # Return value
/// - `Ok(Vec<(String, Vec<String>)>)`: Successfully returns nodes and their topic lists.
/// - `Err(Error)`: Retrieval failed, returns error information.
#[command]
pub async fn get_peers<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, AppState>,
) -> Result<Vec<(String, Vec<String>)>, Error> {
    // Create a oneshot channel
    let (sender, mut receiver) = channel(1);

    // Send GetPeers command, carrying sender
    state
        .command_sender
        .send(Libp2pCommand::GetPeers(sender))
        .await
        .map_err(|e| Error::ChannelSend(e.to_string()))?;

    // Wait for background thread to return result
    receiver
        .recv()
        .await
        .ok_or_else(|| Error::ChannelReceive("Channel closed".to_string()))
}
