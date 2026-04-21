use anyhow::Result;
use gigi_p2p::P2pEvent;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum AppEvent {
    P2P(P2pEvent),
    MessageSaved(String),
    ContactUpdated,
    GroupUpdated,
}

static BROADCAST_TX: Lazy<Mutex<Option<broadcast::Sender<AppEvent>>>> =
    Lazy::new(|| Mutex::new(None));

pub struct EventBus;

impl EventBus {
    pub fn init() {
        println!("Initializing EventBus");
        let (tx, _rx) = broadcast::channel(100);
        *BROADCAST_TX.lock().unwrap() = Some(tx);
        println!("EventBus initialized successfully");
    }

    pub fn send(event: AppEvent) -> Result<usize> {
        println!("EventBus::send called with event: {:?}", event);
        if let Some(tx) = BROADCAST_TX.lock().unwrap().as_ref() {
            println!("EventBus has sender, sending event");
            let result = tx.send(event);
            println!("Event send result: {:?}", result);
            result.map_err(|e| anyhow::anyhow!(e))
        } else {
            println!("EventBus sender not initialized");
            Ok(0)
        }
    }

    pub fn subscribe() -> Option<broadcast::Receiver<AppEvent>> {
        println!("EventBus::subscribe called");
        let result = BROADCAST_TX
            .lock()
            .unwrap()
            .as_ref()
            .map(|tx| tx.subscribe());
        println!("EventBus subscribe result: {:?}", result);
        result
    }
}
