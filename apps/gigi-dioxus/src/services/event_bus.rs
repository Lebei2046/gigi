use anyhow::Result;
use gigi_p2p::P2pEvent;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum AppEvent {
    P2P(P2pEvent),
    MessageSaved(String),
    #[allow(dead_code)]
    ContactUpdated,
    GroupUpdated,
    FileDownloadProgress {
        download_id: String,
        progress: u8,
    },
    FileDownloadCompleted {
        download_id: String,
        path: std::path::PathBuf,
    },
    FileDownloadFailed {
        download_id: String,
        error: String,
    },
    FileShareReceived {
        #[allow(dead_code)]
        from_peer_id: String,
        from_nickname: String,
        share_code: String,
        filename: String,
        file_size: u64,
        file_type: String,
        #[allow(dead_code)]
        conv_id: String,
    },
    GroupFileShareReceived {
        #[allow(dead_code)]
        from_peer_id: String,
        from_nickname: String,
        share_code: String,
        filename: String,
        file_size: u64,
        file_type: String,
        group_name: String,
    },
}

const CHANNEL_CAPACITY: usize = 1000;

static BROADCAST_TX: Lazy<Mutex<Option<broadcast::Sender<AppEvent>>>> =
    Lazy::new(|| Mutex::new(None));

pub struct EventBus;

impl EventBus {
    pub fn init() {
        let (tx, _rx) = broadcast::channel(CHANNEL_CAPACITY);
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
        BROADCAST_TX
            .lock()
            .unwrap()
            .as_ref()
            .map(|tx| tx.subscribe())
    }
}
