//! Offline queue entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "offline_queue")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub message_id: String,
    pub target_nickname: String,
    pub target_peer_id: Option<String>,
    pub queued_at: i64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub last_retry_at: Option<i64>,
    pub next_retry_at: Option<i64>,
    pub expires_at: i64,
    pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::messages::Entity",
        from = "Column::MessageId",
        to = "super::messages::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Message,
}

impl Related<super::messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Message.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
