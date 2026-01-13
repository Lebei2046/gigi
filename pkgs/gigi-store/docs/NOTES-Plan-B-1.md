**According Plan B, design persistence and sync for messages**

我来设计基于方案 B 的消息持久化和同步机制。让我先了解一下现有的代码结构。
基于对现有代码的分析,我现在为您设计**方案 B: 消息队列 + 定期同步**的完整架构:

## 消息持久化和同步设计文档

### 一、核心架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    消息持久化层                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  本地存储     │  │  离线队列     │  │  消息索引     │         │
│  │  (SQLite)    │  │  (未投递)     │  │  (查找优化)   │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    同步协调层                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  1. 检测好友上线 → 触发离线消息推送                        │  │
│  │  2. 定期同步: 每30秒交换未读消息摘要                       │  │
│  │  3. 冲突解决: 基于时间戳和消息ID                           │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    P2P 网络层                                    │
│              (现有 gigi-p2p 架构)                               │
└─────────────────────────────────────────────────────────────────┘
```

### 二、数据结构设计

#### 1. 消息存储模型

```rust
// 存储所有消息的持久化结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,              // UUID v4
    pub msg_type: MessageType,   // Direct/Group
    pub direction: MessageDirection, // Sent/Received
    pub content: MessageContent, // Text/FileShare/ShareGroup
    pub timestamp: u64,          // Unix timestamp (ms)
    pub sender_nickname: String, // 发送者昵称
    pub recipient_nickname: Option<String>, // 单聊接收者
    pub group_name: Option<String>, // 群组名
    pub peer_id: PeerId,         // 发送者 PeerId
    pub delivered: bool,         // 是否已投递
    pub delivered_at: Option<u64>, // 投递时间
    pub read: bool,              // 是否已读
    pub read_at: Option<u64>,    // 已读时间
    pub sync_status: SyncStatus, // 同步状态
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    FileShare {
        share_code: String,
        filename: String,
        file_size: u64,
        file_type: String,
    },
    ShareGroup {
        group_id: String,
        group_name: String,
        inviter_nickname: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Pending,        // 待同步到好友节点
    Synced,         // 已同步
    Delivered,      // 已投递给目标
    Acknowledged,   // 已收到确认
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageDirection {
    Sent,       // 我发送的
    Received,   // 我收到的
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Direct,     // 单聊
    Group,      // 群聊
}
```

#### 2. 离线队列模型

```rust
// 存储需要中继的离线消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineQueueItem {
    pub message_id: String,
    pub target_nickname: String, // 目标昵称
    pub target_peer_id: Option<PeerId>, // 目标 PeerId (上线后填充)
    pub queued_at: u64,          // 入队时间
    pub retry_count: u32,        // 重试次数
    pub last_retry_at: Option<u64>, // 上次重试时间
    pub expires_at: u64,         // 过期时间 (默认7天)
}
```

#### 3. 同步摘要模型

```rust
// 用于定期同步的消息摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSummary {
    pub nickname: String,
    pub last_sync_time: u64,
    pub pending_message_ids: Vec<String>, // 待投递的消息ID
    pub max_message_id: String,           // 本地最大消息ID
}
```

### 三、核心模块设计

#### 模块 1: MessageStore (消息存储层)

```rust
pub struct MessageStore {
    db: SqliteConnection,
    offline_queue: Vec<OfflineQueueItem>,
}

impl MessageStore {
    // 初始化数据库
    pub async fn new(db_path: PathBuf) -> Result<Self>;
    
    // 存储消息
    pub async fn store_message(&mut self, msg: StoredMessage) -> Result<()>;
    
    // 标记消息已投递
    pub async fn mark_delivered(&mut self, message_id: &str) -> Result<()>;
    
    // 标记消息已读
    pub async fn mark_read(&mut self, message_id: &str) -> Result<()>;
    
    // 获取对话历史
    pub async fn get_conversation(
        &self, 
        peer_nickname: &str, 
        limit: usize, 
        offset: usize
    ) -> Result<Vec<StoredMessage>>;
    
    // 获取群组消息
    pub async fn get_group_messages(
        &self, 
        group_name: &str, 
        limit: usize, 
        offset: usize
    ) -> Result<Vec<StoredMessage>>;
    
    // 添加到离线队列
    pub async fn enqueue_offline(&mut self, item: OfflineQueueItem) -> Result<()>;
    
    // 获取待投递消息
    pub async fn get_pending_messages(&self, nickname: &str) -> Result<Vec<StoredMessage>>;
    
    // 清理过期消息
    pub async fn cleanup_expired(&mut self) -> Result<()>;
}
```

#### 模块 2: SyncManager (同步协调层)

```rust
pub struct SyncManager {
    message_store: MessageStore,
    sync_interval: Duration, // 默认30秒
    last_sync: HashMap<String, u64>, // per-peer last sync time
}

impl SyncManager {
    pub fn new(message_store: MessageStore) -> Self;
    
    // 检测到好友上线时触发
    pub async fn on_peer_online(&mut self, nickname: &str, peer_id: PeerId) -> Result<Vec<StoredMessage>>;
    
    // 定期同步: 交换消息摘要
    pub async fn sync_with_peer(&mut self, nickname: &str, peer_id: PeerId) -> Result<SyncSummary>;
    
    // 处理来自好友的同步请求
    pub async fn handle_sync_request(&mut self, summary: SyncSummary) -> Result<SyncSummary>;
    
    // 推送离线消息
    pub async fn push_offline_messages(&mut self, nickname: &str, peer_id: PeerId) -> Result<usize>;
    
    // 确认消息已接收
    pub async fn acknowledge_message(&mut self, message_id: &str) -> Result<()>;
    
    // 定期清理任务
    pub async fn run_cleanup_task(&self);
}
```

#### 模块 3: 扩展的 P2pClient

```rust
pub struct P2pClient {
    // ... 现有字段 ...
    message_store: Arc<Mutex<MessageStore>>,
    sync_manager: Arc<Mutex<SyncManager>>,
}

impl P2pClient {
    pub async fn new_with_persistence(
        keypair: Keypair,
        nickname: String,
        output_dir: PathBuf,
        db_path: PathBuf,
    ) -> Result<(Self, mpsc::UnboundedReceiver<P2pEvent>)>;
    
    // 发送消息(自动持久化)
    pub async fn send_message_persistent(
        &mut self,
        nickname: &str,
        message: String,
    ) -> Result<String>;
    
    // 获取对话历史
    pub async fn get_history(&self, nickname: &str, limit: usize) -> Result<Vec<StoredMessage>>;
    
    // 标记消息已读
    pub async fn mark_as_read(&mut self, message_id: &str) -> Result<()>;
}
```

### 四、数据库 Schema (SQLite)

```sql
-- 消息表
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    msg_type TEXT NOT NULL,         -- 'Direct' or 'Group'
    direction TEXT NOT NULL,         -- 'Sent' or 'Received'
    content_type TEXT NOT NULL,      -- 'Text', 'FileShare', 'ShareGroup'
    content_json TEXT NOT NULL,      -- JSON serialized content
    timestamp INTEGER NOT NULL,      -- Unix timestamp (ms)
    sender_nickname TEXT NOT NULL,
    recipient_nickname TEXT,         -- NULL for group messages
    group_name TEXT,                 -- NULL for direct messages
    peer_id TEXT NOT NULL,
    delivered BOOLEAN DEFAULT FALSE,
    delivered_at INTEGER,
    read BOOLEAN DEFAULT FALSE,
    read_at INTEGER,
    sync_status TEXT DEFAULT 'Pending', -- 'Pending', 'Synced', 'Delivered', 'Acknowledged'
    created_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
);

-- 离线队列表
CREATE TABLE offline_queue (
    message_id TEXT PRIMARY KEY,
    target_nickname TEXT NOT NULL,
    target_peer_id TEXT,             -- NULL initially, set when peer comes online
    queued_at INTEGER NOT NULL,
    retry_count INTEGER DEFAULT 0,
    last_retry_at INTEGER,
    expires_at INTEGER NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

-- 索引优化
CREATE INDEX idx_messages_timestamp ON messages(timestamp DESC);
CREATE INDEX idx_messages_sender ON messages(sender_nickname);
CREATE INDEX idx_messages_recipient ON messages(recipient_nickname);
CREATE INDEX idx_messages_group ON messages(group_name);
CREATE INDEX idx_messages_delivered ON messages(delivered);
CREATE INDEX idx_offline_queue_target ON offline_queue(target_nickname);
CREATE INDEX idx_offline_queue_expires ON offline_queue(expires_at);
```

### 五、同步协议设计

#### 协议 1: OfflineSync (离线消息推送)

```rust
pub enum OfflineSyncMessage {
    // 请求获取离线消息
    RequestPending {
        since: u64, // 上次同步时间戳
    },
    
    // 推送离线消息
    PushMessages {
        messages: Vec<StoredMessage>,
    },
    
    // 确认收到消息
    AckMessages {
        message_ids: Vec<String>,
    },
    
    // 同步摘要
    SyncSummary {
        summary: SyncSummary,
    },
}
```

### 六、集成到现有架构

#### 修改点 1: behaviour.rs

```rust
#[derive(NetworkBehaviour)]
pub struct UnifiedBehaviour {
    pub gigi_dns: GigiDnsBehaviour,
    pub direct_msg: request_response::cbor::Behaviour<DirectMessage, DirectResponse>,
    pub gossipsub: gossipsub::Behaviour,
    pub file_sharing: request_response::cbor::Behaviour<FileSharingRequest, FileSharingResponse>,
    // 新增: 离线同步协议
    pub offline_sync: request_response::cbor::Behaviour<OfflineSyncMessage, OfflineSyncMessage>,
}
```

#### 修改点 2: events.rs

```rust
pub enum P2pEvent {
    // ... 现有事件 ...
    
    // 新增: 离线消息相关事件
    OfflineMessagesReceived {
        from: PeerId,
        from_nickname: String,
        messages: Vec<StoredMessage>,
    },
    OfflineMessagesDelivered {
        to: PeerId,
        to_nickname: String,
        message_ids: Vec<String>,
    },
}
```

### 七、工作流程

#### 流程 1: 发送消息(在线)

```
用户发送消息
    ↓
P2pClient::send_message_persistent()
    ↓
1. 存储到 MessageStore (delivered=false, sync_status=Pending)
    ↓
2. 如果目标在线 → 直接发送
    ↓
3. 发送成功 → 标记 delivered=true, sync_status=Delivered
    ↓
4. 发送失败/离线 → 加入 offline_queue
```

#### 流程 2: 好友上线

```
检测到好友上线 (GigiDnsEvent::PeerDiscovered)
    ↓
SyncManager::on_peer_online()
    ↓
1. 从 offline_queue 获取该好友的待投递消息
    ↓
2. 批量推送消息 (OfflineSyncMessage::PushMessages)
    ↓
3. 等待 AckMessages 确认
    ↓
4. 标记消息 delivered=true, sync_status=Delivered
    ↓
5. 从 offline_queue 移除
```

#### 流程 3: 定期同步

```
每30秒触发
    ↓
SyncManager::sync_with_peer()
    ↓
1. 生成 SyncSummary (pending_message_ids, last_sync_time)
    ↓
2. 发送给好友 (OfflineSyncMessage::SyncSummary)
    ↓
3. 接收好友的 SyncSummary
    ↓
4. 对比本地和远程摘要
    ↓
5. 交换缺失的消息
    ↓
6. 更新 last_sync_time
```

### 八、配置参数

```rust
pub struct SyncConfig {
    // 同步间隔
    pub sync_interval: Duration,         // 默认30秒
    
    // 离线消息过期时间
    pub offline_message_ttl: Duration,   // 默认7天
    
    // 重试配置
    pub max_retry_count: u32,            // 默认10次
    pub retry_interval: Duration,        // 默认5分钟
    
    // 批量推送配置
    pub max_batch_size: usize,           // 默认50条/批
    
    // 清理任务间隔
    pub cleanup_interval: Duration,      // 默认1小时
}
```

### 九、依赖添加

```toml
# pkgs/gigi-p2p/Cargo.toml
[dependencies]
rusqlite = { version = "0.30", features = ["bundled"] }
tokio-rusqlite = "0.5"
uuid = { version = "1.6", features = ["v4", "serde"] }
```

---

这是完整的方案 B 设计。您想让我:

1. **开始实现** MessageStore 和 SyncManager 核心模块?
2. **集成到现有** gigi-p2p 架构中?
3. **先实现一个简化版本** (只支持单聊离线消息)?
4. **调整设计方案** (比如改用其他存储方案)?

请告诉我下一步要做什么!