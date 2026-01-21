//! Conversation entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "conversations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String, // peer-id for direct chats, group-id for groups
    pub name: String,    // peer nickname for direct chat, group name for groups
    pub is_group: bool,  // false = direct chat, true = group chat
    pub peer_id: String, // peer-id for direct chats (same as id), group-id for groups
    pub last_message: Option<String>, // Last message content for preview
    pub last_message_time: Option<i64>, // Last message timestamp
    pub last_message_timestamp: Option<i64>, // For sorting chats
    pub unread_count: i32, // Number of unread messages
    pub created_at: i64, // When conversation was created
    pub updated_at: i64, // When conversation was last updated
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
