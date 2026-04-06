//! Contact manager for storing and managing contact information

use sea_orm::{
    prelude::Expr, ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::entities::contacts;

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub peer_id: String,
    pub name: String,
    pub added_at: i64,
}

impl From<contacts::Model> for ContactInfo {
    fn from(model: contacts::Model) -> Self {
        Self {
            peer_id: model.peer_id,
            name: model.name,
            added_at: model.added_at,
        }
    }
}

/// Contact manager
pub struct ContactManager {
    db: DatabaseConnection,
}

impl ContactManager {
    /// Create a new contact manager
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Add a contact
    pub async fn add(&self, peer_id: &str, name: &str) -> Result<(), DbErr> {
        let now = chrono::Utc::now().timestamp_millis();

        let contact = contacts::ActiveModel {
            peer_id: Set(peer_id.to_string()),
            name: Set(name.to_string()),
            added_at: Set(now),
            ..Default::default()
        };

        // Try to insert, ignoring RecordNotFound errors from the return value
        // but propagating other errors (like constraint violations)
        match contact.insert(&self.db).await {
            Ok(_) => {}
            Err(DbErr::RecordNotFound(_)) => {}
            Err(e) => return Err(e),
        }

        debug!("Added contact: {} ({})", name, peer_id);
        Ok(())
    }

    /// Update a contact's name
    pub async fn update_name(&self, peer_id: &str, name: &str) -> Result<(), DbErr> {
        contacts::Entity::update_many()
            .filter(contacts::Column::PeerId.eq(peer_id))
            .col_expr(contacts::Column::Name, Expr::value(name))
            .exec(&self.db)
            .await?;

        debug!("Updated contact name: {} -> {}", peer_id, name);
        Ok(())
    }

    /// Remove a contact
    pub async fn remove(&self, peer_id: &str) -> Result<(), DbErr> {
        contacts::Entity::delete_many()
            .filter(contacts::Column::PeerId.eq(peer_id))
            .exec(&self.db)
            .await?;

        info!("Removed contact: {}", peer_id);
        Ok(())
    }

    /// Get a contact by peer ID
    pub async fn get(&self, peer_id: &str) -> Result<Option<ContactInfo>, DbErr> {
        let contact = contacts::Entity::find_by_id(peer_id.to_string())
            .one(&self.db)
            .await?;

        Ok(contact.map(|c| c.into()))
    }

    /// Get all contacts
    pub async fn get_all(&self) -> Result<Vec<ContactInfo>, DbErr> {
        let contacts = contacts::Entity::find().all(&self.db).await?;

        Ok(contacts.into_iter().map(|c| c.into()).collect())
    }

    /// Check if a peer ID is already a contact
    pub async fn exists(&self, peer_id: &str) -> Result<bool, DbErr> {
        let count = contacts::Entity::find()
            .filter(contacts::Column::PeerId.eq(peer_id))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_info_serialization() {
        let contact = ContactInfo {
            peer_id: "12D3KooW...".to_string(),
            name: "Alice".to_string(),
            added_at: 1700000000000,
        };

        let json = serde_json::to_string(&contact).unwrap();
        let deserialized: ContactInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.peer_id, contact.peer_id);
        assert_eq!(deserialized.name, contact.name);
        assert_eq!(deserialized.added_at, contact.added_at);
    }
}
