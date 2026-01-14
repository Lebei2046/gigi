//! Shared files entity for storing file sharing information

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "shared_files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub share_code: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub hash: String,
    pub chunk_count: i32,
    pub created_at: i64,
    pub revoked: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
