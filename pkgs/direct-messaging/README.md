# Direct Messaging Library

基于 Rust libp2p 框架的点对点直接消息传递库。

## 特性

- 🔗 **直接 TCP 连接** - 无需 mDNS，支持直接拨号连接
- 💬 **文本消息** - 实时文本消息传递
- 🖼️ **图片传输** - 支持任意图片格式，自动 MIME 类型检测
- 🔐 **端到端加密** - 使用 Noise 协议确保通信安全
- 🚀 **异步处理** - 基于 Tokio 的高性能异步架构
- 📡 **多连接支持** - 同时连接多个节点

## 快速开始

### 安装

```toml
[dependencies]
direct-messaging = { path = "pkgs/direct-messaging" }
libp2p = { version = "0.56", features = ["json"] }
tokio = { version = "1.0", features = ["full"] }
```

### 基本用法

```rust
use direct_messaging::{DirectMessaging, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建消息传递实例
    let (mut messaging, mut event_receiver) = DirectMessaging::new().await?;
    
    // 开始监听连接
    let listen_addr = messaging.start_listening(0)?;
    println!("Listening on: {}", listen_addr);
    
    // 连接到其他节点
    let addr: libp2p::Multiaddr = "/ip4/127.0.0.1/tcp/8080".parse()?;
    messaging.dial_peer(&addr)?;
    
    // 发送消息
    loop {
        tokio::select! {
            event = event_receiver.recv() => {
                if let Some(event) = event {
                    handle_event(event).await;
                }
            }
            // 处理其他逻辑...
        }
    }
}

async fn handle_event(event: direct_messaging::CustomMessagingEvent) {
    match event {
        direct_messaging::CustomMessagingEvent::MessageReceived { from, message } => {
            match message {
                direct_messaging::Message::Text(text) => {
                    println!("收到来自 {} 的文本: {}", from, text);
                }
                direct_messaging::Message::Image { name, mime_type, data } => {
                    println!("收到来自 {} 的图片: {} ({} 字节)", from, name, data.len());
                }
            }
        }
        _ => {}
    }
}
```

## 聊天示例

### 运行示例

```bash
# 启动第一个节点（监听模式）
cargo run --example chat -- --port 8080

# 启动第二个节点（连接模式）
cargo run --example chat -- --addr /ip4/127.0.0.1/tcp/8080
```

### 交互式命令

- `直接输入文本` - 发送文本消息
- `/text <message>` - 发送文本消息
- `/image <path>` - 发送图片文件
- `/connect <multiaddr>` - 连接节点
- `/peers` - 查看连接状态
- `/help` - 帮助信息

### 示例对话

```
Local peer ID: 12D3KooWQVtBYE7zasPLcpkTzs55uo7kDmq3c7EdrH48VxKy2JJG
Listening on: /ip4/0.0.0.0/tcp/8080

✓ Connected to: 12D3KooWJE9WyaRhqyWoDXnwehsgmvULRicLb8kkaxR4EhFKJviT
> hello world
[12D3KooWJE9WyaRhqyWoDXnwehsgmvULRicLb8kkaxR4EhFKJviT] 你好！
> /image ~/screenshot.png
Image 'screenshot.png' sent to 1 peers
[12D3KooWJE9WyaRhqyWoDXnwehsgmvULRicLb8kkaxR4EhFKJviT] Image: screenshot.png (1024567 bytes, image/png)
> /peers
Connected peers (1):
  12D3KooWJE9WyaRhqyWoDXnwehsgmvULRicLb8kkaxR4EhFKJviT
```

## API 参考

### DirectMessaging

主要的消息传递结构体。

#### 方法

- `new()` - 创建新的消息传递实例
- `start_listening(port)` - 开始监听指定端口
- `dial_peer(addr)` - 连接到指定地址的节点
- `send_message(peer_id, message)` - 发送消息到指定节点
- `get_connected_peers()` - 获取所有已连接的节点
- `local_peer_id()` - 获取本地节点 ID

### Message

消息类型枚举。

```rust
pub enum Message {
    Text(String),
    Image {
        name: String,
        mime_type: String,
        data: Vec<u8>,
    },
}
```

#### 构造方法

- `Message::text(content)` - 创建文本消息
- `Message::image(name, mime_type, data)` - 创建图片消息

### CustomMessagingEvent

自定义事件类型，用于接收网络事件。

```rust
pub enum CustomMessagingEvent {
    Connected(PeerId),
    Disconnected(PeerId),
    MessageReceived { from: PeerId, message: Message },
    MessageSent { to: PeerId, message: Message },
    Error(String),
}
```

## 网络协议

### 传输层

- **TCP**: 可靠的传输协议
- **Noise**: 加密握手协议
- **Yamux**: 连接多路复用

### 应用层

- **Request-Response**: 请求-响应模式
- **JSON**: 消息序列化格式
- **协议ID**: `/messaging/1.0.0`

## 安全特性

- **端到端加密**: 使用 Noise 协议
- **身份验证**: Ed25519 密钥对
- **超时保护**: 30秒请求超时
- **连接管理**: 自动清理断开的连接

## 性能优化

- **异步 I/O**: 基于 Tokio 的事件循环
- **流式处理**: 支持大量并发连接
- **内存优化**: 高效的消息缓冲
- **零拷贝**: 减少不必要的数据复制

## 故障排除

### 常见问题

**Q: 连接超时怎么办？**
A: 检查目标地址是否正确，确保目标节点正在监听且网络可达。

**Q: 图片发送失败？**
A: 确保图片文件存在且格式支持，检查文件权限。

**Q: 消息没有收到？**
A: 检查连接状态，使用 `/peers` 命令确认节点已连接。

### 调试

启用详细日志：

```bash
RUST_LOG=debug cargo run --example chat
```

## 许可证

MIT License - 详见项目根目录 LICENSE 文件。