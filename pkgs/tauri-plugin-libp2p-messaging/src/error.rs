/// 定义 libp2p 消息传递插件中的错误类型。
///
/// 该枚举涵盖了插件中可能出现的各种错误，包括 I/O 错误、配置错误、网络错误等。
/// 每个错误变体都提供了详细的错误信息，便于调试和处理。
use serde::{Serialize, Serializer};
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// I/O 操作错误。
  #[error(transparent)]
  Io(#[from] std::io::Error),

  /// Gossipsub 配置错误。
  #[error("Gossipsub config error: {0}")]
  GossipsubConfigError(String),

  /// Gossipsub 协议错误。
  #[error("Gossipsub error: {0}")]
  GossipsubError(String),

  /// mDNS 协议错误。
  #[error("mDNS error: {0}")]
  MdnsError(String),

  /// Swarm 网络错误。
  #[error("Swarm error: {0}")]
  SwarmError(String),

  /// 行为（Behaviour）错误。
  #[error("Behaviour error: {0}")]
  BehaviourError(String),

  /// 订阅错误。
  #[error("Subscription error: {0}")]
  SubscriptionError(String),

  /// 发布错误。
  #[error("Publish error: {0}")]
  PublishError(String),

  /// 锁超时错误。
  #[error("Lock timeout: {0}")]
  LockTimeout(String),

  /// 通道发送错误。
  #[error("Channel send error: {0}")]
  ChannelSend(String),

  /// 通道接收错误。
  #[error("Channel receive error: {0}")]
  ChannelReceive(String),

  /// 通道已关闭错误。
  #[error("Channel closed")]
  ChannelClosed,
}

/// 为 `Error` 实现 `Serialize` trait，使其可以序列化为字符串。
///
/// 主要用于将错误信息转换为 JSON 或其他格式。
impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

/// 从 `SendError<T>` 转换为 `Error`。
///
/// 用于处理通道发送失败时的错误转换。
impl<T> From<SendError<T>> for Error {
  fn from(err: SendError<T>) -> Self {
    Error::ChannelSend(format!("Failed to send command: {}", err))
  }
}
