# Tauri Plugin libp2p-messaging

Messaging over p2p, with mDNS and gossip.

## For development:

- bun install
- cargo build
- bun run build

## 核心代码逻辑分析

### Rust 部分 ( src/lib.rs )

**功能概述：**

这是一个基于 libp2p 和 Tauri 的消息传递插件，支持发布/订阅模式和节点发现。主要功能包括：

- 订阅/取消订阅主题。
- 发布消息到指定主题。
- 发现和管理网络中的其他节点（Peer）。

**关键组件：**

- Libp2pMessagingBehaviour ：

结合了 gossipsub （用于发布/订阅消息）和 mdns （用于节点发现）的行为。

- Libp2pMessaging ：
  
  - 管理 Swarm （libp2p 的网络堆栈）。
  - 提供订阅、取消订阅、发布消息和获取节点列表的接口。
  - 事件处理：监听 gossipsub 的消息事件和 mdns 的节点发现事件，并通过 Tauri 的 emit 方法通知前端。
  
- 初始化：

通过 init 函数初始化插件，并注册 Tauri 命令处理器（如订阅、发布等）。

### TypeScript 部分 ( guest-js/index.ts )


```javascript
import { onMessageReceived, onPeerDiscovered } from './index';

// 监听消息接收事件
const unlistenMessage = await onMessageReceived(({ topic, data, sender }) => {
  console.log(`收到消息：主题=${topic}, 内容=${data}, 发送者=${sender}`);
});

// 监听节点发现事件
const unlistenPeer = await onPeerDiscovered(({ id, addresses }) => {
  console.log(`发现节点：ID=${id}, 地址=${addresses.join(', ')}`);
});

// 取消监听
// unlistenMessage();
// unlistenPeer();
```

## 总结

项目的核心逻辑集中在 Rust 部分，实现了基于 libp2p 的分布式消息传递功能，并通过 Tauri 插件的形式暴露给前端调用。


