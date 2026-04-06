我理解了!既然 OctoBase 还在积极开发中,依赖它可能会带来稳定性问题。让我重新设计方案,基于**方案 B (消息队列 + 定期同步)**,但使用更稳定的技术栈。
好的,让我基于**方案 B**设计一个更稳定的实现,使用成熟的技术栈:

## 方案 B: 消息队列 + 定期同步 (改进版)

### 一、技术栈选择

```toml
# 成熟、稳定的依赖
[dependencies]
# 数据库: SQLCipher (加密 SQLite)
rusqlite = { version = "0.30", features = ["bundled", "chrono"] }

# 异步支持
tokio = { version = "1.35", features = ["full"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# UUID
uuid = { version = "1.6", features = ["v4", "serde"] }

# 时间处理
chrono = { version = "0.4", features = ["serde"] }

# 加密 (可选,如果需要加密消息)
aes-gcm = "0.10"

# 压缩 (减少同步数据量)
zstd = "0.13"
```

### 二、数据库 Schema 设计

```sql
-- 消息表 (存储所有聊天消息)
CREATE TABLE messages (
    id TEXT PRIMARY KEY,                    -- UUID v4
    msg_type TEXT NOT NULL,                 -- 'Direct' | 'Group'
    direction TEXT NOT NULL,               -- 'Sent' | 'Received'
    content_type TEXT NOT NULL,             -- 'Text' | 'FileShare' | 'ShareGroup'
    content_json TEXT NOT NULL,             -- JSON serialized content
    sender_nickname TEXT NOT NULL,
    recipient_nickname TEXT,                -- NULL for group messages
    group_name TEXT,                        -- NULL for direct messages
    peer_id TEXT NOT NULL,                  -- 发送者 PeerId
    timestamp INTEGER NOT NULL,             -- Unix timestamp (ms)
    created_at INTEGER NOT NULL,
    
    -- 投递状态
    delivered BOOLEAN DEFAULT FALSE,
    delivered_at INTEGER,
    
    -- 已读状态
    read BOOLEAN DEFAULT FALSE,
    read_at INTEGER,
    
    -- 同步状态
    sync_status TEXT DEFAULT 'Pending',    -- 'Pending' | 'Synced' | 'Delivered' | 'Acknowledged'
    sync_attempts INTEGER DEFAULT 0,
    last_sync_attempt INTEGER,
    
    -- 过期时间 (7天后自动清理)
    expires_at INTEGER NOT NULL
);

-- 离线队列表 (待投递给离线好友的消息)
CREATE TABLE offline_queue (
    message_id TEXT PRIMARY KEY,
    target_nickname TEXT NOT NULL,
    target_peer_id TEXT,                    -- NULL initially, set when peer comes online
    queued_at INTEGER NOT NULL,
    
    -- 重试机制
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 10,
    last_retry_at INTEGER,
    next_retry_at INTEGER,
    
    -- 过期时间
    expires_at INTEGER NOT NULL,
    
    -- 状态
    status TEXT DEFAULT 'Pending',          -- 'Pending' | 'InProgress' | 'Delivered' | 'Expired'
    
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

-- 消息确认表 (跟踪哪些节点已确认收到消息)
CREATE TABLE message_acknowledgments (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    acknowledged_by_nickname TEXT NOT NULL,
    acknowledged_by_peer_id TEXT NOT NULL,
    acknowledged_at INTEGER NOT NULL,
    ack_type TEXT NOT NULL,                 -- 'Delivered' | 'Read'
    
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

-- 索引优化
CREATE INDEX idx_messages_timestamp ON messages(timestamp DESC);
CREATE INDEX idx_messages_sender ON messages(sender_nickname);
CREATE INDEX idx_messages_recipient ON messages(recipient_nickname);
CREATE INDEX idx_messages_group ON messages(group_name);
CREATE INDEX idx_messages_sync_status ON messages(sync_status);
CREATE INDEX idx_messages_expires ON messages(expires_at);

CREATE INDEX idx_offline_queue_target ON offline_queue(target_nickname);
CREATE INDEX idx_offline_queue_status ON offline_queue(status);
CREATE INDEX idx_offline_queue_next_retry ON offline_queue(next_retry_at);
CREATE INDEX idx_offline_queue_expires ON offline_queue(expires_at);

CREATE INDEX idx_acknowledgments_message ON message_acknowledgments(message_id);
CREATE INDEX idx_acknowledgments_by_peer ON message_acknowledgments(acknowledged_by_nickname);
```

### 三、数据结构定义

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Direct,
    Group,
}

/// 消息方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageDirection {
    Sent,
    Received,
}

/// 消息内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    Text { text: String },
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

/// 同步状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Pending,        // 待同步
    Synced,         // 已同步到好友节点
    Delivered,      // 已投递给目标
    Acknowledged,   // 已收到确认
}

/// 存储的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub msg_type: MessageType,
    pub direction: MessageDirection,
    pub content: MessageContent,
    pub sender_nickname: String,
    pub recipient_nickname: Option<String>,
    pub group_name: Option<String>,
    pub peer_id: String,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    
    pub delivered: bool,
    pub delivered_at: Option<DateTime<Utc>>,
    
    pub read: bool,
    pub read_at: Option<DateTime<Utc>>,
    
    pub sync_status: SyncStatus,
    pub sync_attempts: u32,
    pub last_sync_attempt: Option<DateTime<Utc>>,
    
    pub expires_at: DateTime<Utc>,
}

/// 离线队列项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineQueueItem {
    pub message_id: String,
    pub target_nickname: String,
    pub target_peer_id: Option<String>,
    pub queued_at: DateTime<Utc>,
    
    pub retry_count: u32,
    pub max_retries: u32,
    pub last_retry_at: Option<DateTime<Utc>>,
    pub next_retry_at: DateTime<Utc>,
    
    pub expires_at: DateTime<Utc>,
    pub status: QueueStatus,
}

/// 队列状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueueStatus {
    Pending,
    InProgress,
    Delivered,
    Expired,
}

/// 消息确认
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAcknowledgment {
    pub id: String,
    pub message_id: String,
    pub acknowledged_by_nickname: String,
    pub acknowledged_by_peer_id: String,
    pub acknowledged_at: DateTime<Utc>,
    pub ack_type: AckType,
}

/// 确认类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AckType {
    Delivered,
    Read,
}
```

### 四、核心模块实现

#### 模块 1: MessageStore (消息存储层)

```rust
use rusqlite::{Connection, params, Row};
use tokio::sync::RwLock;
use std::path::PathBuf;
use std::collections::HashMap;

pub struct MessageStore {
    db: Arc<RwLock<Connection>>,
    local_nickname: String,
}

impl MessageStore {
    /// 初始化数据库
    pub async fn new(db_path: PathBuf, local_nickname: String) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        
        // 创建表
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                msg_type TEXT NOT NULL,
                direction TEXT NOT NULL,
                content_type TEXT NOT NULL,
                content_json TEXT NOT NULL,
                sender_nickname TEXT NOT NULL,
                recipient_nickname TEXT,
                group_name TEXT,
                peer_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                delivered BOOLEAN DEFAULT FALSE,
                delivered_at INTEGER,
                read BOOLEAN DEFAULT FALSE,
                read_at INTEGER,
                sync_status TEXT DEFAULT 'Pending',
                sync_attempts INTEGER DEFAULT 0,
                last_sync_attempt INTEGER,
                expires_at INTEGER NOT NULL
            )
            "#,
            [],
        )?;
        
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS offline_queue (
                message_id TEXT PRIMARY KEY,
                target_nickname TEXT NOT NULL,
                target_peer_id TEXT,
                queued_at INTEGER NOT NULL,
                retry_count INTEGER DEFAULT 0,
                max_retries INTEGER DEFAULT 10,
                last_retry_at INTEGER,
                next_retry_at INTEGER,
                expires_at INTEGER NOT NULL,
                status TEXT DEFAULT 'Pending',
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
            )
            "#,
            [],
        )?;
        
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS message_acknowledgments (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                acknowledged_by_nickname TEXT NOT NULL,
                acknowledged_by_peer_id TEXT NOT NULL,
                acknowledged_at INTEGER NOT NULL,
                ack_type TEXT NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
            )
            "#,
            [],
        )?;
        
        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_recipient ON messages(recipient_nickname)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_sync_status ON messages(sync_status)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_offline_queue_target ON offline_queue(target_nickname)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_offline_queue_next_retry ON offline_queue(next_retry_at)",
            [],
        )?;
        
        Ok(Self {
            db: Arc::new(RwLock::new(conn)),
            local_nickname,
        })
    }
    
    /// 存储消息
    pub async fn store_message(
        &self,
        msg: StoredMessage,
    ) -> Result<()> {
        let db = self.db.write().await;
        
        db.execute(
            r#"
            INSERT INTO messages (
                id, msg_type, direction, content_type, content_json,
                sender_nickname, recipient_nickname, group_name, peer_id,
                timestamp, created_at, delivered, delivered_at,
                read, read_at, sync_status, sync_attempts,
                last_sync_attempt, expires_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                    ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
            "#,
            params![
                msg.id,
                serde_json::to_string(&msg.msg_type)?,
                serde_json::to_string(&msg.direction)?,
                serde_json::to_string(&msg.content)?,
                serde_json::to_string(&msg.content)?,
                msg.sender_nickname,
                msg.recipient_nickname,
                msg.group_name,
                msg.peer_id,
                msg.timestamp.timestamp_millis(),
                msg.created_at.timestamp_millis(),
                msg.delivered,
                msg.delivered_at.map(|t| t.timestamp_millis()),
                msg.read,
                msg.read_at.map(|t| t.timestamp_millis()),
                serde_json::to_string(&msg.sync_status)?,
                msg.sync_attempts,
                msg.last_sync_attempt.map(|t| t.timestamp_millis()),
                msg.expires_at.timestamp_millis(),
            ],
        )?;
        
        Ok(())
    }
    
    /// 添加到离线队列
    pub async fn enqueue_offline(
        &self,
        message_id: String,
        target_nickname: String,
    ) -> Result<()> {
        let db = self.db.write().await;
        
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(7);
        let next_retry_at = now + chrono::Duration::minutes(5);
        
        db.execute(
            r#"
            INSERT INTO offline_queue (
                message_id, target_nickname, queued_at,
                retry_count, max_retries, last_retry_at,
                next_retry_at, expires_at, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                message_id,
                target_nickname,
                now.timestamp_millis(),
                0u32,
                10u32,
                Option::<i64>::None,
                next_retry_at.timestamp_millis(),
                expires_at.timestamp_millis(),
                "Pending",
            ],
        )?;
        
        Ok(())
    }
    
    /// 获取待投递的消息
    pub async fn get_pending_messages(
        &self,
        target_nickname: &str,
        limit: usize,
    ) -> Result<Vec<StoredMessage>> {
        let db = self.db.read().await;
        
        let mut stmt = db.prepare(
            r#"
            SELECT m.* FROM messages m
            INNER JOIN offline_queue q ON m.id = q.message_id
            WHERE q.target_nickname = ?1 AND q.status = 'Pending'
            ORDER BY m.timestamp ASC
            LIMIT ?2
            "#,
        )?;
        
        let messages = stmt
            .query_map(params![target_nickname, limit], |row| {
                self.row_to_stored_message(row)
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        
        Ok(messages)
    }
    
    /// 获取对话历史
    pub async fn get_conversation(
        &self,
        peer_nickname: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<StoredMessage>> {
        let db = self.db.read().await;
        
        let mut stmt = db.prepare(
            r#"
            SELECT * FROM messages
            WHERE (sender_nickname = ?1 OR recipient_nickname = ?1)
            ORDER BY timestamp DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )?;
        
        let messages = stmt
            .query_map(params![peer_nickname, limit, offset], |row| {
                self.row_to_stored_message(row)
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        
        Ok(messages)
    }
    
    /// 标记消息已投递
    pub async fn mark_delivered(&self, message_id: &str) -> Result<()> {
        let db = self.db.write().await;
        
        let now = Utc::now().timestamp_millis();
        
        // 更新消息状态
        db.execute(
            "UPDATE messages SET delivered = TRUE, delivered_at = ?1, sync_status = 'Delivered' WHERE id = ?2",
            params![now, message_id],
        )?;
        
        // 更新队列状态
        db.execute(
            "UPDATE offline_queue SET status = 'Delivered' WHERE message_id = ?1",
            params![message_id],
        )?;
        
        Ok(())
    }
    
    /// 标记消息已读
    pub async fn mark_read(&self, message_id: &str) -> Result<()> {
        let db = self.db.write().await;
        
        let now = Utc::now().timestamp_millis();
        
        db.execute(
            "UPDATE messages SET read = TRUE, read_at = ?1 WHERE id = ?2",
            params![now, message_id],
        )?;
        
        Ok(())
    }
    
    /// 获取需要重试的消息
    pub async fn get_retry_messages(&self, limit: usize) -> Result<Vec<(String, String)>> {
        let db = self.db.read().await;
        
        let mut stmt = db.prepare(
            r#"
            SELECT message_id, target_nickname FROM offline_queue
            WHERE status = 'Pending' AND next_retry_at <= ?1
            AND retry_count < max_retries
            ORDER BY next_retry_at ASC
            LIMIT ?2
            "#,
        )?;
        
        let items = stmt
            .query_map(params![Utc::now().timestamp_millis(), limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                ))
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        
        Ok(items)
    }
    
    /// 更新重试信息
    pub async fn update_retry(
        &self,
        message_id: &str,
        success: bool,
    ) -> Result<()> {
        let db = self.db.write().await;
        
        let now = Utc::now();
        
        if success {
            db.execute(
                "UPDATE offline_queue SET status = 'Delivered' WHERE message_id = ?1",
                params![message_id],
            )?;
        } else {
            // 增加重试次数,计算下次重试时间 (指数退避: 5, 10, 20, 40, 80, 160, 320, 640, 1280, 2560 分钟)
            db.execute(
                r#"
                UPDATE offline_queue 
                SET retry_count = retry_count + 1,
                    last_retry_at = ?1,
                    next_retry_at = ?2
                WHERE message_id = ?3
                "#,
                params![
                    now.timestamp_millis(),
                    (now + chrono::Duration::minutes(5 * 2_i64.pow(10) as i64)).timestamp_millis(),
                    message_id,
                ],
            )?;
        }
        
        Ok(())
    }
    
    /// 清理过期消息
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let db = self.db.write().await;
        
        let now = Utc::now().timestamp_millis();
        
        // 清理过期的离线队列项
        let queue_count = db.execute(
            "DELETE FROM offline_queue WHERE expires_at < ?1 OR (retry_count >= max_retries AND next_retry_at < ?1)",
            params![now],
        )?;
        
        // 清理过期的消息 (只清理已投递且超过7天的)
        let msg_count = db.execute(
            "DELETE FROM messages WHERE expires_at < ?1 AND delivered = TRUE",
            params![now],
        )?;
        
        Ok((queue_count + msg_count) as u64)
    }
    
    /// 获取未读消息数
    pub async fn get_unread_count(&self, peer_nickname: &str) -> Result<u64> {
        let db = self.db.read().await;
        
        let count: u64 = db.query_row(
            "SELECT COUNT(*) FROM messages WHERE sender_nickname = ?1 AND read = FALSE",
            params![peer_nickname],
            |row| row.get(0),
        )?;
        
        Ok(count)
    }
    
    /// 辅助函数: 从 Row 构造 StoredMessage
    fn row_to_stored_message(&self, row: &Row) -> Result<StoredMessage> {
        Ok(StoredMessage {
            id: row.get(0)?,
            msg_type: serde_json::from_str(&row.get::<_, String>(1)?)?,
            direction: serde_json::from_str(&row.get::<_, String>(2)?)?,
            content: serde_json::from_str(&row.get::<_, String>(4)?)?,
            sender_nickname: row.get(5)?,
            recipient_nickname: row.get(6)?,
            group_name: row.get(7)?,
            peer_id: row.get(8)?,
            timestamp: DateTime::from_timestamp_millis(row.get(9)?)?.with_timezone(&Utc),
            created_at: DateTime::from_timestamp_millis(row.get(10)?)?.with_timezone(&Utc),
            delivered: row.get(11)?,
            delivered_at: row.get::<_, Option<i64>>(12)?
                .map(|t| DateTime::from_timestamp_millis(t)?.with_timezone(&Utc)),
            read: row.get(13)?,
            read_at: row.get::<_, Option<i64>>(14)?
                .map(|t| DateTime::from_timestamp_millis(t)?.with_timezone(&Utc)),
            sync_status: serde_json::from_str(&row.get::<_, String>(15)?)?,
            sync_attempts: row.get(16)?,
            last_sync_attempt: row.get::<_, Option<i64>>(17)?
                .map(|t| DateTime::from_timestamp_millis(t)?.with_timezone(&Utc)),
            expires_at: DateTime::from_timestamp_millis(row.get(18)?)?.with_timezone(&Utc),
        })
    }
}
```

#### 模块 2: SyncManager (同步协调层)

```rust
use libp2p::PeerId;
use tokio::sync::mpsc;
use std::time::Duration;
use std::collections::HashMap;

pub struct SyncManager {
    message_store: Arc<MessageStore>,
    pending_syncs: Arc<RwLock<HashMap<String, SyncState>>>,
    sync_interval: Duration,
    retry_interval: Duration,
    cleanup_interval: Duration,
}

#[derive(Debug, Clone)]
struct SyncState {
    last_sync: DateTime<Utc>,
    in_progress: bool,
}

impl SyncManager {
    pub fn new(
        message_store: Arc<MessageStore>,
        sync_interval: Duration,
    ) -> Self {
        Self {
            message_store,
            pending_syncs: Arc::new(RwLock::new(HashMap::new())),
            sync_interval,
            retry_interval: Duration::from_secs(300), // 5分钟
            cleanup_interval: Duration::from_secs(3600), // 1小时
        }
    }
    
    /// 检测到好友上线
    pub async fn on_peer_online(
        &self,
        nickname: &str,
        peer_id: PeerId,
    ) -> Result<Vec<StoredMessage>> {
        tracing::info!("Peer {} ({}) came online", nickname, peer_id);
        
        // 获取待投递的消息
        let messages = self.message_store.get_pending_messages(nickname, 50).await?;
        
        if messages.is_empty() {
            tracing::debug!("No pending messages for {}", nickname);
            return Ok(vec![]);
        }
        
        tracing::info!("Found {} pending messages for {}", messages.len(), nickname);
        
        // 更新同步状态
        let mut syncs = self.pending_syncs.write().await;
        syncs.insert(nickname.to_string(), SyncState {
            last_sync: Utc::now(),
            in_progress: true,
        });
        
        Ok(messages)
    }
    
    /// 标记同步完成
    pub async fn on_sync_complete(
        &self,
        nickname: &str,
        delivered_count: usize,
    ) -> Result<()> {
        tracing::info!("Sync with {} completed, {} messages delivered", nickname, delivered_count);
        
        let mut syncs = self.pending_syncs.write().await;
        if let Some(state) = syncs.get_mut(nickname) {
            state.last_sync = Utc::now();
            state.in_progress = false;
        }
        
        Ok(())
    }
    
    /// 定期同步任务
    pub async fn run_sync_task<F>(
        &self,
        mut sync_callback: F,
    ) where
        F: Fn(String, Vec<StoredMessage>) -> std::pin::Pin<Box<dyn Future<Output = Result<usize>> + Send>> + Send + 'static,
    {
        let mut sync_interval = tokio::time::interval(self.sync_interval);
        let mut retry_interval = tokio::time::interval(self.retry_interval);
        let mut cleanup_interval = tokio::time::interval(self.cleanup_interval);
        
        loop {
            tokio::select! {
                _ = sync_interval.tick() => {
                    // 定期同步所有在线好友
                    self.sync_all_online_peers(&mut sync_callback).await;
                }
                
                _ = retry_interval.tick() => {
                    // 重试失败的消息
                    self.retry_failed_messages(&mut sync_callback).await;
                }
                
                _ = cleanup_interval.tick() => {
                    // 清理过期消息
                    match self.message_store.cleanup_expired().await {
                        Ok(count) => {
                            if count > 0 {
                                tracing::info!("Cleaned up {} expired messages", count);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to cleanup expired messages: {}", e);
                        }
                    }
                }
            }
        }
    }
    
    async fn sync_all_online_peers<F>(&self, sync_callback: &mut F)
    where
        F: Fn(String, Vec<StoredMessage>) -> std::pin::Pin<Box<dyn Future<Output = Result<usize>> + Send>> + Send + 'static,
    {
        // 获取所有有待同步消息的好友
        let db = self.message_store.db.read().await;
        let mut stmt = db.prepare(
            "SELECT DISTINCT target_nickname FROM offline_queue WHERE status = 'Pending'"
        ).ok();
        
        if let Some(mut stmt) = stmt {
            let peers = stmt.query_map([], |row| row.get::<_, String>(0))
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();
            
            drop(db);
            
            for nickname in peers {
                // 检查是否正在同步
                let syncs = self.pending_syncs.read().await;
                if let Some(state) = syncs.get(&nickname) {
                    if state.in_progress {
                        continue;
                    }
                }
                drop(syncs);
                
                // 获取待同步消息
                match self.message_store.get_pending_messages(&nickname, 50).await {
                    Ok(messages) => {
                        if !messages.is_empty() {
                            tracing::info!("Syncing {} messages to {}", messages.len(), nickname);
                            
                            // 执行同步回调 (通过 P2P 发送)
                            let nickname_clone = nickname.clone();
                            let callback_result = sync_callback(nickname_clone, messages).await;
                            
                            match callback_result {
                                Ok(delivered) => {
                                    let _ = self.on_sync_complete(&nickname, delivered).await;
                                }
                                Err(e) => {
                                    tracing::error!("Failed to sync to {}: {}", nickname, e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to get pending messages for {}: {}", nickname, e);
                    }
                }
            }
        }
    }
    
    async fn retry_failed_messages<F>(&self, sync_callback: &mut F)
    where
        F: Fn(String, Vec<StoredMessage>) -> std::pin::Pin<Box<dyn Future<Output = Result<usize>> + Send>> + Send + 'static,
    {
        // 获取需要重试的消息
        match self.message_store.get_retry_messages(100).await {
            Ok(items) => {
                if !items.is_empty() {
                    tracing::info!("Retrying {} messages", items.len());
                    
                    // 按目标分组
                    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
                    for (message_id, nickname) in items {
                        grouped.entry(nickname).or_default().push(message_id);
                    }
                    
                    for (nickname, message_ids) in grouped {
                        // 获取完整消息
                        let mut messages = Vec::new();
                        for message_id in message_ids {
                            // TODO: 实现根据 ID 获取消息的方法
                        }
                        
                        if !messages.is_empty() {
                            let nickname_clone = nickname.clone();
                            let callback_result = sync_callback(nickname_clone, messages).await;
                            
                            match callback_result {
                                Ok(delivered) => {
                                    let _ = self.on_sync_complete(&nickname, delivered).await;
                                }
                                Err(e) => {
                                    tracing::error!("Retry failed for {}: {}", nickname, e);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to get retry messages: {}", e);
            }
        }
    }
}
```

这个设计方案使用成熟、稳定的技术栈:

1. **SQLite + Rusqlite**: 经过充分测试的嵌入式数据库
2. **Tokio**: 成熟的异步运行时
3. **Serde**: Rust 生态标准的序列化框架
4. **Chrono**: 稳定的时间处理库

您想让我继续实现:
1. **完整的 P2pClient 集成**?
2. **同步协议实现**?
3. **测试用例**?
4. **还是调整设计方案**?