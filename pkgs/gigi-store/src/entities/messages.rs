//! Message entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub msg_type: String,     // "Direct" or "Group"
    pub direction: String,    // "Sent" or "Received"
    pub content_type: String, // "Text" or "FileShare"
    pub content_json: String, // Full content as JSON
    pub sender_nickname: String,
    pub recipient_nickname: Option<String>,
    pub group_name: Option<String>,
    pub peer_id: String,
    pub timestamp: i64,
    pub created_at: i64,
    pub delivered: bool,
    pub delivered_at: Option<i64>,
    pub read: bool,
    pub read_at: Option<i64>,
    pub sync_status: String, // "Pending", "Synced", "Delivered", or "Acknowledged"
    pub sync_attempts: u32,
    pub last_sync_attempt: Option<i64>,
    pub expires_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::offline_queue::Entity")]
    OfflineQueue,
    #[sea_orm(has_many = "super::message_acknowledgments::Entity")]
    MessageAcknowledgments,
}

impl Related<super::offline_queue::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OfflineQueue.def()
    }
}

impl Related<super::message_acknowledgments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MessageAcknowledgments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
