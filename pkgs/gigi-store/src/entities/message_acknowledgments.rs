//! Message acknowledgment entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "message_acknowledgments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub message_id: String,
    pub acknowledged_by_nickname: String,
    pub acknowledged_by_peer_id: String,
    pub acknowledged_at: i64,
    pub ack_type: String,
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
