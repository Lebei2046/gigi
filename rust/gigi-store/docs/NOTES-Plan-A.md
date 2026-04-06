我来设计基于方案 A (CRDT) 的消息持久化和同步架构。首先让我了解一下 OctoBase 的 CRDT 能力。
完美!现在基于 OctoBase 的 CRDT 能力,我来设计**方案 A: 基于 CRDT 的离线同步**的完整架构。

## 方案 A: 基于 CRDT 的消息持久化和同步设计

### 一、核心架构

```
┌─────────────────────────────────────────────────────────────────┐
│              OctoBase CRDT Layer (消息队列)                        │
│                                                                  │
│  Workspace: "gigi-messages"                                       │
│  ├── Space: "direct-{peer-nickname}" (单聊队列)                   │
│  │   ├── Block: "messages" [flavor="message-array"]               │
│  │   │   └── Array<Message> (CRDT ORArray)                        │
│  │   ├── Block: "metadata" [flavor="queue-metadata"]              │
│  │   │   ├── last_read_index: u64                                 │
│  │   │   ├── delivered_index: u64                                 │
│  │   │   └── sync_timestamp: u64                                  │
│  │   └── Block: "acknowledgments" [flavor="ack-array"]            │
│  │       └── Array<Acknowledgment>                                 │
│  │                                                                │
│  └── Space: "group-{group-name}" (群聊队列)                        │
│      └── (结构同单聊)                                              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ CRDT Sync
┌─────────────────────────────────────────────────────────────────┐
│              CRDT 同步协调层                                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  1. State Vector 同步: 交换已知的更新状态                    │   │
│  │  2. Update 推送: 实时同步 CRDT 更新                          │   │
│  │  3. Awareness: 跟踪节点在线状态和消费进度                     │   │
│  │  4. 冲突解决: CRDT 自动合并,无需额外逻辑                       │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              P2P 传输层 (现有 gigi-p2p)                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │  CRDT Sync   │  │  Awareness   │  │  Direct Msg │           │
│  │  Protocol    │  │  Protocol    │  │  (通知层)    │           │
│  └──────────────┘  └──────────────┘  └──────────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

### 二、CRDT 数据结构设计

#### 1. 消息 CRDT 结构

```rust
// OctoBase Block Flavor: "message"
pub struct MessageBlock {
    pub block_id: String,        // UUID v4
    pub sender_nickname: String,  // 发送者昵称
    pub sender_peer_id: String,   // 发送者 PeerId (字符串存储)
    pub content_type: String,     // "text" | "file-share" | "share-group"
    pub content: Any,            // 消息内容 (Any 类型,支持 JSON 序列化)
    pub timestamp: i64,          // Unix timestamp (ms)
    pub created_at: i64,          // Block 创建时间
    pub status: String,          // "pending" | "delivered" | "read"
    pub delivered_at: Option<i64>,
    pub read_at: Option<i64>,
}

// 消息内容的 Any 类型
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

// 转换为 Any (OctoBase 兼容)
impl MessageContent {
    pub fn to_any(&self) -> Any {
        match self {
            MessageContent::Text(text) => Any::Object(HashMap::from([
                ("type".to_string(), Any::String("text".to_string())),
                ("text".to_string(), Any::String(text.clone())),
            ])),
            MessageContent::FileShare { share_code, filename, file_size, file_type } => {
                Any::Object(HashMap::from([
                    ("type".to_string(), Any::String("file-share".to_string())),
                    ("share_code".to_string(), Any::String(share_code.clone())),
                    ("filename".to_string(), Any::String(filename.clone())),
                    ("file_size".to_string(), Any::BigInt64(*file_size as i64)),
                    ("file_type".to_string(), Any::String(file_type.clone())),
                ]))
            }
            MessageContent::ShareGroup { group_id, group_name, inviter_nickname } => {
                Any::Object(HashMap::from([
                    ("type".to_string(), Any::String("share-group".to_string())),
                    ("group_id".to_string(), Any::String(group_id.clone())),
                    ("group_name".to_string(), Any::String(group_name.clone())),
                    ("inviter_nickname".to_string(), Any::String(inviter_nickname.clone())),
                ]))
            }
        }
    }
    
    pub fn from_any(any: &Any) -> Result<Self> {
        let obj = any.as_object().ok_or(Error::InvalidMessageFormat)?;
        let msg_type = obj.get("type")
            .and_then(|v| v.as_string())
            .ok_or(Error::MissingMessageType)?;
        
        match msg_type.as_str() {
            "text" => {
                let text = obj.get("text")
                    .and_then(|v| v.as_string())
                    .ok_or(Error::MissingContent)?;
                Ok(MessageContent::Text(text.clone()))
            }
            "file-share" => {
                let share_code = obj.get("share_code")
                    .and_then(|v| v.as_string()).ok_or(Error::MissingShareCode)?;
                let filename = obj.get("filename")
                    .and_then(|v| v.as_string()).ok_or(Error::MissingFilename)?;
                let file_size = obj.get("file_size")
                    .and_then(|v| v.as_i64()).ok_or(Error::MissingFileSize)?;
                let file_type = obj.get("file_type")
                    .and_then(|v| v.as_string()).ok_or(Error::MissingFileType)?;
                Ok(MessageContent::FileShare {
                    share_code: share_code.clone(),
                    filename: filename.clone(),
                    file_size: file_size as u64,
                    file_type: file_type.clone(),
                })
            }
            "share-group" => {
                let group_id = obj.get("group_id")
                    .and_then(|v| v.as_string()).ok_or(Error::MissingGroupId)?;
                let group_name = obj.get("group_name")
                    .and_then(|v| v.as_string()).ok_or(Error::MissingGroupName)?;
                let inviter_nickname = obj.get("inviter_nickname")
                    .and_then(|v| v.as_string()).ok_or(Error::MissingInviter)?;
                Ok(MessageContent::ShareGroup {
                    group_id: group_id.clone(),
                    group_name: group_name.clone(),
                    inviter_nickname: inviter_nickname.clone(),
                })
            }
            _ => Err(Error::UnknownMessageType(msg_type)),
        }
    }
}
```

#### 2. 队列元数据 CRDT

```rust
// OctoBase Block Flavor: "queue-metadata"
pub struct QueueMetadata {
    pub last_read_index: u64,      // 已读消息索引 (Array 索引)
    pub delivered_index: u64,      // 已投递索引
    pub sync_timestamp: u64,       // 最后同步时间
    pub participant_nicknames: Vec<String>, // 参与者昵称列表
}

// 存储在 Block 的 properties 中
impl QueueMetadata {
    pub fn from_block(block: &Block) -> Result<Self> {
        Ok(Self {
            last_read_index: block.get("last_read_index")
                .and_then(|v| v.to_any()?.as_i64())
                .map(|i| i as u64)
                .unwrap_or(0),
            delivered_index: block.get("delivered_index")
                .and_then(|v| v.to_any()?.as_i64())
                .map(|i| i as u64)
                .unwrap_or(0),
            sync_timestamp: block.get("sync_timestamp")
                .and_then(|v| v.to_any()?.as_i64())
                .map(|i| i as u64)
                .unwrap_or(0),
            participant_nicknames: block.get("participant_nicknames")
                .and_then(|v| v.to_any()?.as_array())
                .map(|arr| arr.iter().filter_map(|a| a.as_string()).collect())
                .unwrap_or_default(),
        })
    }
}
```

#### 3. 确认消息 CRDT

```rust
// OctoBase Block Flavor: "acknowledgment"
pub struct Acknowledgment {
    pub message_index: u64,       // 消息在 Array 中的索引
    pub acknowledged_by: String,  // 确认者昵称
    pub acknowledged_at: u64,     // 确认时间戳
    pub ack_type: String,         // "delivered" | "read"
}
```

### 三、核心模块设计

#### 模块 1: CrdtMessageStore (CRDT 消息存储层)

```rust
use jwst_storage::JwstStorage;
use jwst_core::{Doc, Space, Block};
use jwst_codec::DocMessage;

pub struct CrdtMessageStore {
    storage: JwstStorage,
    workspace_id: String,
    local_nickname: String,
    local_peer_id: PeerId,
    // 缓存: space_id -> (space, doc)
    spaces: Arc<RwLock<HashMap<String, (Space, Arc<Doc>)>>>,
}

impl CrdtMessageStore {
    /// 初始化 CRDT 消息存储
    pub async fn new(
        db_path: PathBuf,
        workspace_id: String,
        local_nickname: String,
        local_peer_id: PeerId,
    ) -> Result<Self> {
        let storage = JwstStorage::new_with_migration(
            &format!("sqlite:{}", db_path.display()),
            BlobStorageType::DB,
        ).await?;
        
        // 创建或获取工作空间
        storage.create_workspace(&workspace_id).await?;
        
        Ok(Self {
            storage,
            workspace_id,
            local_nickname,
            local_peer_id,
            spaces: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// 获取或创建单聊空间
    pub async fn get_or_create_direct_space(
        &self,
        peer_nickname: &str,
    ) -> Result<(Space, Arc<Doc>)> {
        let space_id = format!("direct-{}", peer_nickname);
        let mut spaces = self.spaces.write().await;
        
        if let Some((space, doc)) = spaces.get(&space_id) {
            return Ok((space.clone(), doc.clone()));
        }
        
        // 从存储加载工作空间
        let workspace = self.storage.get_workspace(&self.workspace_id).await?;
        let mut space = workspace.get_space(&space_id)?;
        
        // 创建或获取消息数组 Block
        if space.get("messages").is_none() {
            let messages_block = space.create("messages", "message-array")?;
            // 确保这个 Block 有 children 数组
            // OctoBase 的 Block 自动有 children 属性
        }
        
        // 创建或获取元数据 Block
        if space.get("metadata").is_none() {
            let metadata_block = space.create("metadata", "queue-metadata")?;
            metadata_block.set("last_read_index", 0i64)?;
            metadata_block.set("delivered_index", 0i64)?;
            metadata_block.set("sync_timestamp", 0i64)?;
            metadata_block.set("participant_nicknames", Any::Array(vec![
                Any::String(self.local_nickname.clone()),
                Any::String(peer_nickname.to_string()),
            ]))?;
        }
        
        // 创建确认 Block
        if space.get("acknowledgments").is_none() {
            space.create("acknowledgments", "ack-array")?;
        }
        
        let doc = space.doc();
        spaces.insert(space_id.clone(), (space.clone(), Arc::new(doc.clone())));
        
        Ok((space, Arc::new(doc)))
    }
    
    /// 获取或创建群组空间
    pub async fn get_or_create_group_space(
        &self,
        group_name: &str,
    ) -> Result<(Space, Arc<Doc>)> {
        let space_id = format!("group-{}", group_name);
        // ... 类似单聊空间
    }
    
    /// 添加消息到 CRDT 队列
    pub async fn append_message(
        &self,
        peer_nickname: &str,
        content: MessageContent,
    ) -> Result<String> {
        let (space, doc) = self.get_or_create_direct_space(peer_nickname).await?;
        let messages_block = space.get("messages").ok_or(Error::MessagesBlockNotFound)?;
        let mut children = messages_block.children();
        
        // 创建消息 Block
        let message_id = Uuid::new_v4().to_string();
        let mut message_block = space.create(&message_id, "message")?;
        
        // 设置消息属性
        message_block.set("sender_nickname", self.local_nickname.clone())?;
        message_block.set("sender_peer_id", self.local_peer_id.to_string())?;
        message_block.set("content_type", content.get_type())?;
        message_block.set("content", content.to_any())?;
        let now = Utc::now().timestamp_millis();
        message_block.set("timestamp", now)?;
        message_block.set("created_at", now)?;
        message_block.set("status", "pending")?;
        message_block.set("delivered_at", Any::Null)?;
        message_block.set("read_at", Any::Null)?;
        
        // 添加到消息数组
        children.push(message_id.clone())?;
        
        // 持久化到存储
        self.storage.full_migrate(
            self.workspace_id.clone(),
            None,
            false,
        ).await?;
        
        Ok(message_id)
    }
    
    /// 获取消息历史 (支持分页)
    pub async fn get_messages(
        &self,
        peer_nickname: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MessageBlock>> {
        let (space, _doc) = self.get_or_create_direct_space(peer_nickname).await?;
        let messages_block = space.get("messages").ok_or(Error::MessagesBlockNotFound)?;
        let children = messages_block.children();
        
        // 获取所有消息 Block ID
        let mut message_ids: Vec<String> = children.iter()
            .filter_map(|v| v.to_any().ok()?.as_string())
            .collect();
        
        // 倒序 (最新的在前)
        message_ids.reverse();
        
        // 分页
        let start = offset;
        let end = (offset + limit).min(message_ids.len());
        let page_ids = &message_ids[start..end];
        
        // 加载消息 Blocks
        let mut messages = Vec::new();
        for block_id in page_ids {
            if let Some(block) = space.get(block_id) {
                if let Ok(msg) = MessageBlock::from_block(&block) {
                    messages.push(msg);
                }
            }
        }
        
        Ok(messages)
    }
    
    /// 标记消息已投递
    pub async fn mark_delivered(
        &self,
        peer_nickname: &str,
        message_index: u64,
    ) -> Result<()> {
        let (space, doc) = self.get_or_create_direct_space(peer_nickname).await?;
        let messages_block = space.get("messages").ok_or(Error::MessagesBlockNotFound)?;
        let children = messages_block.children();
        
        // 获取指定索引的消息
        if let Some(msg_id) = children.get(message_index as usize)
            .and_then(|v| v.to_any().ok()?.as_string()) {
            if let Some(block) = space.get(&msg_id) {
                block.set("status", "delivered")?;
                block.set("delivered_at", Utc::now().timestamp_millis())?;
            }
        }
        
        // 更新元数据
        let metadata_block = space.get("metadata").ok_or(Error::MetadataBlockNotFound)?;
        metadata_block.set("delivered_index", message_index as i64)?;
        
        self.storage.full_migrate(
            self.workspace_id.clone(),
            None,
            false,
        ).await?;
        
        Ok(())
    }
    
    /// 标记消息已读
    pub async fn mark_read(
        &self,
        peer_nickname: &str,
        message_index: u64,
    ) -> Result<()> {
        let (space, _doc) = self.get_or_create_direct_space(peer_nickname).await?;
        let messages_block = space.get("messages").ok_or(Error::MessagesBlockNotFound)?;
        let children = messages_block.children();
        
        // 获取消息
        if let Some(msg_id) = children.get(message_index as usize)
            .and_then(|v| v.to_any().ok()?.as_string()) {
            if let Some(block) = space.get(&msg_id) {
                block.set("status", "read")?;
                block.set("read_at", Utc::now().timestamp_millis())?;
            }
        }
        
        // 更新元数据
        let metadata_block = space.get("metadata").ok_or(Error::MetadataBlockNotFound)?;
        metadata_block.set("last_read_index", message_index as i64)?;
        
        self.storage.full_migrate(
            self.workspace_id.clone(),
            None,
            false,
        ).await?;
        
        Ok(())
    }
    
    /// 订阅 CRDT 变更
    pub async fn subscribe<F>(
        &self,
        peer_nickname: &str,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(Vec<u8>, Vec<History>) + Send + Sync + 'static,
    {
        let (_space, doc) = self.get_or_create_direct_space(peer_nickname).await?;
        doc.subscribe(callback);
        Ok(())
    }
    
    /// 生成 State Vector (用于差分同步)
    pub fn get_state_vector(&self, peer_nickname: &str) -> Result<Vec<u8>> {
        let doc = self.spaces.read()
            .get(&format!("direct-{}", peer_nickname))
            .map(|(_, d)| d.clone())
            .ok_or(Error::SpaceNotFound)?;
        
        Ok(doc.get_state_vector())
    }
    
    /// 编码 State Vector 之后的更新
    pub fn encode_state_as_update(
        &self,
        peer_nickname: &str,
        state_vector: &[u8],
    ) -> Result<Vec<u8>> {
        let doc = self.spaces.read()
            .get(&format!("direct-{}", peer_nickname))
            .map(|(_, d)| d.clone())
            .ok_or(Error::SpaceNotFound)?;
        
        let sv = StateVector::decode_v1(state_vector)
            .map_err(|_| Error::InvalidStateVector)?;
        Ok(doc.encode_state_as_update_v1(&sv)?)
    }
    
    /// 应用远程更新
    pub fn apply_update(&self, peer_nickname: &str, update: Vec<u8>) -> Result<()> {
        let doc = self.spaces.read()
            .get(&format!("direct-{}", peer_nickname))
            .map(|(_, d)| d.clone())
            .ok_or(Error::SpaceNotFound)?;
        
        doc.apply_update_from_binary_v1(update)?;
        Ok(())
    }
}
```

#### 模块 2: CrdtSyncManager (CRDT 同步协调层)

```rust
pub struct CrdtSyncManager {
    message_store: Arc<CrdtMessageStore>,
    sync_interval: Duration,
    last_sync_with_peer: Arc<RwLock<HashMap<String, u64>>>,
    awareness: Arc<Awareness>,
}

impl CrdtSyncManager {
    pub fn new(
        message_store: Arc<CrdtMessageStore>,
        sync_interval: Duration,
        local_client_id: u64,
    ) -> Self {
        Self {
            message_store,
            sync_interval,
            last_sync_with_peer: Arc::new(RwLock::new(HashMap::new())),
            awareness: Arc::new(Awareness::new(local_client_id)),
        }
    }
    
    /// 检测到好友上线时触发同步
    pub async fn on_peer_online(&mut self, nickname: &str, peer_id: PeerId) -> Result<SyncAction> {
        // 1. 获取该好友的 State Vector
        let my_sv = self.message_store.get_state_vector(nickname)?;
        
        // 2. 请求好友的 State Vector
        Ok(SyncAction::RequestStateVector {
            target: peer_id,
            target_nickname: nickname.to_string(),
            my_state_vector: my_sv,
        })
    }
    
    /// 处理来自好友的 State Vector 请求
    pub async fn handle_state_vector_request(
        &self,
        peer_id: PeerId,
        peer_nickname: &str,
        peer_state_vector: Vec<u8>,
        my_state_vector: Vec<u8>,
    ) -> Result<SyncAction> {
        // 1. 生成我需要发送的更新 (基于对方的 State Vector)
        let updates_to_send = self.message_store
            .encode_state_as_update(peer_nickname, &peer_state_vector)?;
        
        // 2. 保存对方的 State Vector 用于后续差分同步
        let mut last_sync = self.last_sync_with_peer.write().await;
        last_sync.insert(peer_nickname.to_string(), Utc::now().timestamp_millis() as u64);
        drop(last_sync);
        
        // 3. 返回需要发送给对方的更新
        Ok(SyncAction::SendUpdates {
            target: peer_id,
            target_nickname: peer_nickname.to_string(),
            updates: updates_to_send,
        })
    }
    
    /// 应用来自好友的更新
    pub async fn apply_remote_updates(
        &self,
        peer_nickname: &str,
        updates: Vec<u8>,
    ) -> Result<Vec<MessageBlock>> {
        // 应用更新到 CRDT
        self.message_store.apply_update(peer_nickname, updates)?;
        
        // 获取新增的消息 (通过订阅回调或比较 State Vector)
        // 这里简化实现:获取最近的 10 条消息
        self.message_store.get_messages(peer_nickname, 10, 0).await
    }
    
    /// 定期同步任务
    pub async fn run_sync_task(&self) {
        let mut interval = tokio::time::interval(self.sync_interval);
        
        loop {
            interval.tick().await;
            
            // 遍历所有在线好友,触发同步
            let peers = self.get_online_peers().await;
            
            for (nickname, peer_id) in peers {
                if let Ok(action) = self.on_peer_online(&nickname, peer_id).await {
                    // 执行同步操作
                    // 这里需要与 P2P 层集成
                    self.execute_sync_action(action).await;
                }
            }
        }
    }
    
    /// Awareness: 跟踪节点在线状态
    pub fn update_peer_status(&self, peer_id: PeerId, nickname: &str, online: bool) {
        let mut states = self.awareness.get_states();
        
        if online {
            // 设置在线状态
            let state = HashMap::from([
                ("nickname".to_string(), Any::String(nickname.to_string())),
                ("status".to_string(), Any::String("online".to_string())),
                ("last_seen".to_string(), Any::BigInt64(Utc::now().timestamp())),
            ]);
            states.insert(peer_id.to_string(), state);
        } else {
            // 移除状态
            states.remove(&peer_id.to_string());
        }
    }
    
    /// Awareness: 订阅状态变更
    pub fn subscribe_awareness<F>(&self, callback: F)
    where
        F: Fn(&Awareness, AwarenessEvent) + Send + Sync + 'static,
    {
        self.awareness.on_update(callback);
    }
    
    async fn get_online_peers(&self) -> Vec<(String, PeerId)> {
        // 从 awareness 获取在线节点
        let states = self.awareness.get_states();
        states.iter()
            .filter_map(|(peer_id, state)| {
                let nickname = state.get("nickname")?.as_string()?;
                let status = state.get("status")?.as_string()?;
                if status == "online" {
                    Some((nickname.clone(), PeerId::from_str(peer_id).ok()?))
                } else {
                    None
                }
            })
            .collect()
    }
    
    async fn execute_sync_action(&self, action: SyncAction) {
        // 实现: 通过 P2P 层发送同步消息
        match action {
            SyncAction::RequestStateVector { target, target_nickname, my_state_vector } => {
                // 发送 CRDT 同步协议消息
                // 这部分将在 P2P 集成中实现
            }
            SyncAction::SendUpdates { target, updates, .. } => {
                // 发送 CRDT 更新
            }
        }
    }
}

#[derive(Debug)]
pub enum SyncAction {
    RequestStateVector {
        target: PeerId,
        target_nickname: String,
        my_state_vector: Vec<u8>,
    },
    SendUpdates {
        target: PeerId,
        target_nickname: String,
        updates: Vec<u8>,
    },
}
```

#### 模块 3: CRDT 同步协议

```rust
/// CRDT 同步协议消息 (在 gigi-p2p 中新增)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtSyncMessage {
    /// Step 1: 发送我的 State Vector
    Step1 {
        peer_nickname: String,
        space_id: String, // "direct-{nickname}" or "group-{name}"
        state_vector: Vec<u8>,
    },
    
    /// Step 2: 基于 State Vector 发送需要的更新
    Step2 {
        peer_nickname: String,
        space_id: String,
        updates: Vec<u8>,
    },
    
    /// Update: 持续的增量更新
    Update {
        peer_nickname: String,
        space_id: String,
        updates: Vec<u8>,
    },
    
    /// Awareness: 节点在线状态
    Awareness {
        client_id: u64,
        state: HashMap<String, Any>, // nickname, status, last_seen
    },
    
    /// Awareness Query: 请求所有节点的在线状态
    AwarenessQuery,
}

pub struct CrdtSyncHandler {
    message_store: Arc<CrdtMessageStore>,
    sync_manager: Arc<CrdtSyncManager>,
}

impl CrdtSyncHandler {
    pub fn new(
        message_store: Arc<CrdtMessageStore>,
        sync_manager: Arc<CrdtSyncManager>,
    ) -> Self {
        Self {
            message_store,
            sync_manager,
        }
    }
    
    pub async fn handle_message(
        &mut self,
        from_peer: PeerId,
        message: CrdtSyncMessage,
        event_sender: &mut mpsc::UnboundedSender<P2pEvent>,
    ) -> Result<Option<CrdtSyncMessage>> {
        match message {
            CrdtSyncMessage::Step1 { peer_nickname, space_id, state_vector } => {
                // 对方发送了 State Vector,我需要返回他们需要的更新
                let my_sv = self.message_store.get_state_vector(&peer_nickname)?;
                let updates = self.message_store
                    .encode_state_as_update(&peer_nickname, &state_vector)?;
                
                Ok(Some(CrdtSyncMessage::Step2 {
                    peer_nickname: self.sync_manager.get_local_nickname().await,
                    space_id,
                    updates,
                }))
            }
            
            CrdtSyncMessage::Step2 { peer_nickname, space_id, updates } => {
                // 对方发送了更新,应用并通知新消息
                let new_messages = self.sync_manager
                    .apply_remote_updates(&peer_nickname, updates).await?;
                
                // 发送事件给应用层
                for msg in new_messages {
                    event_sender.unbounded_send(P2pEvent::DirectMessage {
                        from: from_peer,
                        from_nickname: peer_nickname.clone(),
                        message: msg.content.to_text()?,
                    })?;
                }
                
                Ok(None)
            }
            
            CrdtSyncMessage::Update { peer_nickname, space_id, updates } => {
                // 实时增量更新
                let new_messages = self.sync_manager
                    .apply_remote_updates(&peer_nickname, updates).await?;
                
                for msg in new_messages {
                    event_sender.unbounded_send(P2pEvent::DirectMessage {
                        from: from_peer,
                        from_nickname: peer_nickname.clone(),
                        message: msg.content.to_text()?,
                    })?;
                }
                
                Ok(None)
            }
            
            CrdtSyncMessage::Awareness { client_id, state } => {
                // 更新 awareness
                if let Some(nickname) = state.get("nickname").and_then(|v| v.as_string()) {
                    let online = state.get("status")
                        .and_then(|v| v.as_string())
                        .map(|s| s == "online")
                        .unwrap_or(false);
                    
                    self.sync_manager.update_peer_status(from_peer, nickname, online);
                }
                Ok(None)
            }
            
            CrdtSyncMessage::AwarenessQuery => {
                // 返回我的 awareness 状态
                let local_nickname = self.sync_manager.get_local_nickname().await;
                let state = HashMap::from([
                    ("nickname".to_string(), Any::String(local_nickname)),
                    ("status".to_string(), Any::String("online".to_string())),
                    ("last_seen".to_string(), Any::BigInt64(Utc::now().timestamp())),
                ]);
                
                Ok(Some(CrdtSyncMessage::Awareness {
                    client_id: self.sync_manager.get_local_client_id().await,
                    state,
                }))
            }
        }
    }
}
```

### 四、集成到 gigi-p2p

#### 1. behaviour.rs - 添加 CRDT 同步行为

```rust
// 在 behaviour.rs 中添加
use crate::crdt_sync::CrdtSyncMessage;

#[derive(NetworkBehaviour)]
pub struct UnifiedBehaviour {
    pub gigi_dns: GigiDnsBehaviour,
    pub direct_msg: request_response::cbor::Behaviour<DirectMessage, DirectResponse>,
    pub gossipsub: gossipsub::Behaviour,
    pub file_sharing: request_response::cbor::Behaviour<FileSharingRequest, FileSharingResponse>,
    
    // 新增: CRDT 同步协议
    pub crdt_sync: request_response::cbor::Behaviour<CrdtSyncMessage, CrdtSyncMessage>,
}

// UnifiedEvent 添加 CRDT 事件
pub enum UnifiedEvent {
    GigiDns(gigi_dns::GigiDnsEvent),
    DirectMessage(request_response::Event<DirectMessage, DirectResponse>),
    Gossipsub(gossipsub::Event),
    FileSharing(request_response::Event<FileSharingRequest, FileSharingResponse>),
    
    // 新增
    CrdtSync(request_response::Event<CrdtSyncMessage, CrdtSyncMessage>),
}
```

#### 2. events.rs - 添加持久化相关事件

```rust
pub enum P2pEvent {
    // ... 现有事件 ...
    
    // 新增: CRDT 消息事件
    OfflineMessagesReceived {
        from: PeerId,
        from_nickname: String,
        messages: Vec<MessageBlock>,
    },
    
    // 新增: 好友上线/离线事件
    PeerStatusChanged {
        peer_id: PeerId,
        nickname: String,
        online: bool,
    },
    
    // 新增: 消息同步进度
    SyncProgress {
        peer_nickname: String,
        synced_count: usize,
        total_count: usize,
    },
}
```

#### 3. client/p2p_client.rs - 集成 CRDT 存储

```rust
pub struct P2pClient {
    // ... 现有字段 ...
    
    // 新增: CRDT 消息存储
    message_store: Option<Arc<CrdtMessageStore>>,
    sync_manager: Option<Arc<CrdtSyncManager>>,
}

impl P2pClient {
    /// 创建带持久化的 P2P 客户端
    pub async fn new_with_persistence(
        keypair: Keypair,
        nickname: String,
        output_dir: PathBuf,
        db_path: PathBuf, // CRDT 数据库路径
    ) -> Result<(Self, mpsc::UnboundedReceiver<P2pEvent>)> {
        // 创建 CRDT 存储
        let workspace_id = format!("gigi-messages-{}", nickname);
        let message_store = Arc::new(CrdtMessageStore::new(
            db_path,
            workspace_id,
            nickname.clone(),
            local_peer_id,
        ).await?);
        
        let sync_manager = Arc::new(CrdtSyncManager::new(
            message_store.clone(),
            Duration::from_secs(30),
            local_client_id,
        ));
        
        // ... 其余初始化 ...
        
        Ok((client, event_receiver))
    }
    
    /// 发送持久化消息
    pub async fn send_message_persistent(
        &mut self,
        nickname: &str,
        message: String,
    ) -> Result<String> {
        if let Some(store) = &self.message_store {
            // 添加到 CRDT
            let message_id = store.append_message(
                nickname,
                MessageContent::Text(message),
            ).await?;
            
            // 如果目标在线,直接发送通知
            if let Some(peer_id) = self.peer_manager.get_peer_id_by_nickname(nickname).ok() {
                // 发送通知: "有新消息"
                // 实际消息通过 CRDT 同步
                self.send_notification(peer_id, "new_message").await?;
            } else {
                // 目标离线:消息已在 CRDT 中,上线后会自动同步
                tracing::debug!("Peer {} offline, message stored in CRDT", nickname);
            }
            
            Ok(message_id)
        } else {
            // 回退到非持久化模式
            self.send_direct_message(nickname, message)
        }
    }
    
    /// 获取消息历史
    pub async fn get_history(
        &self,
        nickname: &str,
        limit: usize,
    ) -> Result<Vec<MessageBlock>> {
        if let Some(store) = &self.message_store {
            store.get_messages(nickname, limit, 0).await
        } else {
            Err(Error::PersistenceNotEnabled)
        }
    }
    
    /// 标记消息已读
    pub async fn mark_as_read(&mut self, nickname: &str, message_index: u64) -> Result<()> {
        if let Some(store) = &self.message_store {
            store.mark_read(nickname, message_index).await?;
            
            // 通过 CRDT 同步更新
            if let Some(peer_id) = self.peer_manager.get_peer_id_by_nickname(nickname).ok() {
                self.send_read_receipt(peer_id, message_index).await?;
            }
            
            Ok(())
        } else {
            Err(Error::PersistenceNotEnabled)
        }
    }
}
```

### 五、依赖添加

```toml
# pkgs/gigi-p2p/Cargo.toml
[dependencies]
# OctoBase CRDT
jwst-core = { path = "../octobase/libs/jwst-core" }
jwst-codec = { path = "../octobase/libs/jwst-codec" }
jwst-storage = { path = "../octobase/libs/jwst-storage" }
jwst-rpc = { path = "../octobase/libs/jwst-rpc" }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# UUID
uuid = { version = "1.6", features = ["v4", "serde"] }

# 时间
chrono = "0.4"
```

### 六、工作流程

#### 流程 1: 发送消息(在线)

```
用户发送 "Hello"
    ↓
P2pClient::send_message_persistent("Alice", "Hello")
    ↓
CrdtMessageStore::append_message()
    ├─ 创建消息 Block (id: msg-123)
    ├─ 设置: sender="Me", content="Hello", status="pending"
    ├─ 添加到 messages Block 的 children 数组
    └─ 持久化到 JwstStorage (SQLite)
    ↓
检查 "Alice" 是否在线
    ├─ 在线 → 发送通知 "new_message" (通过 DirectMsg)
    │         Alice 收到通知 → 触发 CRDT 同步 → 获取新消息
    └─ 离线 → 消息已在 CRDT 中,等待上线同步
```

#### 流程 2: 好友上线同步

```
Alice 上线 (GigiDnsEvent::PeerDiscovered)
    ↓
CrdtSyncManager::on_peer_online("Alice")
    ↓
1. 获取我的 State Vector (对 Alice 的 Space)
    ↓
2. 发送 CrdtSyncMessage::Step1 {
       peer_nickname: "Me",
       space_id: "direct-Alice",
       state_vector: <我已知的更新>
   }
    ↓
Alice 收到 Step1
    ↓
Alice::encode_state_as_update(my_state_vector)
    ├─ 基于 Me 的 State Vector
    └─ 生成 Me 需要的更新
    ↓
Alice 发送 CrdtSyncMessage::Step2 {
       peer_nickname: "Alice",
       space_id: "direct-Alice",
       updates: <离线消息的 CRDT 更新>
   }
    ↓
Me 收到 Step2
    ↓
Me::apply_remote_updates(updates)
    ├─ 应用 CRDT 更新到本地 Space
    ├─ 检测到新消息 (通过订阅或比较)
    └─ 触发 P2pEvent::OfflineMessagesReceived
    ↓
应用层显示离线消息 "Hello"
```

#### 流程 3: 实时同步(双方在线)

```
双方都已建立 CRDT 同步连接
    ↓
Me 发送新消息 → 添加到 CRDT
    ↓
CrdtStore 订阅回调触发 (CRDT 变更)
    ↓
自动编码更新: doc.encode_update()
    ↓
发送 CrdtSyncMessage::Update { updates: <增量更新> }
    ↓
Alice 实时收到更新 → 应用到 CRDT → 显示消息
```

### 七、优势与特点

1. **离线消息自动同步**: 消息存储在 CRDT 中,好友上线时自动同步
2. **无中心服务器**: 纯 P2P,CRDT 状态在所有参与者间同步
3. **冲突解决**: CRDT 自动处理并发编辑,无需手动冲突解决
4. **部分同步**: 基于 State Vector 的差分同步,减少数据传输
5. **实时更新**: 订阅机制实时推送增量更新
6. **持久化**: 自动持久化到 SQLite,重启后数据不丢失
7. **可扩展**: 支持单聊、群聊,结构一致
8. **CloudBase 可选**: 可将 Workspace 同步到 CloudBase 作为"种子节点",确保数据不丢失

---

这是基于 OctoBase CRDT 的完整设计方案。您想让我:

1. **开始实现** CrdtMessageStore 核心模块?
2. **集成到 gigi-p2p** 的现有架构?
3. **实现简化版本** (只支持单聊)?
4. **调整设计** (比如添加 CloudBase 作为备份)?

请告诉我下一步!