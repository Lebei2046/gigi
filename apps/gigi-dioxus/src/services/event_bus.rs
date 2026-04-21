
use gigi_p2p::P2pEvent;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::sync::broadcast;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum AppEvent {
    P2P(P2pEvent),
    MessageSaved(String),
    ContactUpdated,
    GroupUpdated,
}

static BROADCAST_TX: Lazy<Mutex<Option<broadcast::Sender<AppEvent>>>> = Lazy::new(|| Mutex::new(None));

pub struct EventBus;

impl EventBus {
    pub fn init() {
        let (tx, _rx) = broadcast::channel(100);
        *BROADCAST_TX.lock().unwrap() = Some(tx);
    }

    pub fn send(event: AppEvent) -> Result<usize> {
        if let Some(tx) = BROADCAST_TX.lock().unwrap().as_ref() {
            tx.send(event).map_err(|e| anyhow::anyhow!(e))
        } else {
            Ok(0)
        }
    }

    pub fn subscribe() -> Option<broadcast::Receiver<AppEvent>> {
        BROADCAST_TX.lock().unwrap().as_ref().map(|tx| tx.subscribe())
    }
}
