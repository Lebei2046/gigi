use hex;
use libp2p::identity;

#[tauri::command]
fn try_get_peer_id(priv_key: &str) -> Result<String, String> {
    match hex::decode(priv_key) {
        Ok(bytes) => match identity::Keypair::ed25519_from_bytes(bytes) {
            Ok(id_keys) => {
                let peer_id = id_keys.public().to_peer_id();
                Ok(peer_id.to_string())
            }
            Err(e) => Err(format!("Failed to create keypair: {}", e)),
        },
        Err(e) => Err(format!("Failed to decode private key: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![try_get_peer_id])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
