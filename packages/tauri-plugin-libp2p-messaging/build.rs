const COMMANDS: &[&str] = &[
  "subscribe_topic",
  "unsubscribe_topic",
  "send_message",
  "get_peers",
];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
