

Gigi：基于 P2P 网络的社交应用
====

本项目为一个基于 P2P 网络的社交应用，结合了 Rust 和 TypeScript/React 技术栈，使用 Tauri 进行多端开发。

## 项目结构概览

- `apps/`：前端应用目录
  - `gigi-mobile`：Gigi 移动端应用
- `pkgs/`：后端核心库目录
  - `direct-messaging`：基于 libp2p 的直接消息传递库
  - `gigi-messaging`：一个 Tauri 后端，用于实现基于 libp2p 的消息传递功能

## Direct Messaging 库

### 简介

`pkgs/direct-messaging` 是一个基于 Rust libp2p 框架的点对点消息传递库，支持：

- 🔗 直接 TCP 连接（无需 mDNS）
- 💬 文本消息传递
- 🖼️ 图片文件传输
- 🔐 Noise 协议加密
- 🚀 高性能异步处理

### 快速开始

#### 安装依赖

```bash
# 安装项目依赖
bun install

# 构建 Rust 库
cargo build --package direct-messaging
```

#### 运行聊天示例

1. **启动第一个节点（监听模式）**：
   ```bash
   cargo run --example chat -- --port 8080
   ```

2. **启动第二个节点（连接模式）**：
   ```bash
   cargo run --example chat -- --addr /ip4/127.0.0.1/tcp/8080
   ```

### 交互式命令

启动聊天应用后，可以使用以下命令：

- `直接输入文本` - 发送文本消息到所有连接的节点
- `/text <message>` - 发送文本消息到所有连接的节点
- `/image <path>` - 发送图片文件到所有连接的节点
- `/connect <multiaddr>` - 连接到指定节点
- `/peers` - 查看已连接的节点
- `/help` - 显示帮助信息

### 示例用法

```bash
# 节点 1：启动监听
cargo run --example chat -- --port 8080

# 节点 2：连接并发送消息
cargo run --example chat -- --addr /ip4/127.0.0.1/tcp/8080

# 在聊天界面中：
> /connect /ip4/127.0.0.1/tcp/8081
> hello world
> /image /path/to/image.jpg
> /peers
```

### API 使用

```rust
use direct_messaging::{DirectMessaging, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建消息传递实例
    let (mut messaging, _receiver) = DirectMessaging::new().await?;
    
    // 开始监听
    let listen_addr = messaging.start_listening(8080)?;
    println!("Listening on: {}", listen_addr);
    
    // 连接到其他节点
    let addr: libp2p::Multiaddr = "/ip4/127.0.0.1/tcp/8081".parse()?;
    messaging.dial_peer(&addr)?;
    
    // 发送文本消息
    let peers = messaging.get_connected_peers();
    for peer_id in peers {
        messaging.send_message(peer_id, Message::text("Hello!")).await?;
    }
    
    Ok(())
}
```

## 开发与构建

### Direct Messaging 库

```bash
# 构建库
cargo build --package direct-messaging

# 运行测试
cargo test --package direct-messaging

# 运行示例
cargo run --example chat --package direct-messaging
```

### Gigi 主应用

#### 移动开发

1. 确保你已安装 Rust 和 Tauri CLI。
2. 进入 `apps/gigi-mobile` 目录
    - bun install
    - bun run tauri dev, or
    - bun run tauri android dev
3. 构建
    - bun run tauri build, or
    - bun run tauri android build

## 技术特性

### libp2p 网络层

- **TCP 传输层**：稳定的 TCP 连接
- **Noise 加密**：端到端加密通信
- **Yamux 多路复用**：单一连接上的多路通信
- **Request-Response 协议**：可靠的请求-响应模式
- **JSON 编解码**：高效的序列化/反序列化

### 消息类型

- **Text**：纯文本消息
- **Image**：图片文件（自动 MIME 类型检测）

### 安全特性

- Ed25519 密钥对生成
- Noise 协议加密
- 连接超时保护
- 消息确认机制

## 服务条款

详见 `apps/gigi-mobile/src/assets/TermsOfUse.md`。

## 如何贡献

欢迎贡献代码，提交 Issue 或 Pull Request。请遵循项目代码规范，并确保提交的代码通过测试。

## 许可证

本项目遵循 MIT 许可证。详见根目录下的 `LICENSE` 文件。
