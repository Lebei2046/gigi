//! Group manager for storing and managing group information

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::entities::groups;

/// Group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub name: String,
    pub joined: bool,
    pub created_at: i64,
}

impl From<groups::Model> for GroupInfo {
    fn from(model: groups::Model) -> Self {
        Self {
            group_id: model.group_id,
            name: model.name,
            joined: model.joined,
            created_at: model.created_at,
        }
    }
}

/// Group manager for storing and managing group information
pub struct GroupManager {
    db: DatabaseConnection,
}

impl GroupManager {
    /// Create a new group manager
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Add or update a group
    pub async fn add_or_update(
        &self,
        group_id: &str,
        name: &str,
        joined: bool,
    ) -> Result<(), DbErr> {
        debug!("Adding/updating group: {}", group_id);

        let existing = groups::Entity::find()
            .filter(groups::Column::GroupId.eq(group_id))
            .one(&self.db)
            .await?;

        if let Some(model) = existing {
            // Update existing group
            let mut active_model: groups::ActiveModel = model.into();
            active_model.name = Set(name.to_string());
            active_model.joined = Set(joined);
            active_model.update(&self.db).await?;
            info!("Group '{}' updated successfully", group_id);
        } else {
            // Insert new group
            let now = chrono::Utc::now().timestamp_millis();
            let new_group = groups::ActiveModel {
                group_id: Set(group_id.to_string()),
                name: Set(name.to_string()),
                joined: Set(joined),
                created_at: Set(now),
            };

            // Use insert without trying to fetch the inserted model
            match new_group.insert(&self.db).await {
                Ok(_) => info!("Group '{}' created successfully", group_id),
                Err(e) => {
                    // Check if it's a RecordNotFound error and the group might actually exist
                    if e.to_string().contains("RecordNotFound") {
                        // Try to query the group to see if it actually exists
                        if let Ok(Some(_)) = groups::Entity::find()
                            .filter(groups::Column::GroupId.eq(group_id))
                            .one(&self.db)
                            .await
                        {
                            info!("Group '{}' already exists, treating as success", group_id);
                            return Ok(());
                        }
                    }
                    tracing::error!("Failed to insert group '{}': {:?}", group_id, e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Get a group by group_id
    pub async fn get(&self, group_id: &str) -> Result<Option<GroupInfo>, DbErr> {
        debug!("Getting group: {}", group_id);

        let result = groups::Entity::find()
            .filter(groups::Column::GroupId.eq(group_id))
            .one(&self.db)
            .await?;

        Ok(result.map(GroupInfo::from))
    }

    /// Get all groups
    pub async fn get_all(&self) -> Result<Vec<GroupInfo>, DbErr> {
        let groups = groups::Entity::find().all(&self.db).await?;

        Ok(groups.into_iter().map(GroupInfo::from).collect())
    }

    /// Get all joined groups
    pub async fn get_joined(&self) -> Result<Vec<GroupInfo>, DbErr> {
        let groups = groups::Entity::find()
            .filter(groups::Column::Joined.eq(true))
            .all(&self.db)
            .await?;

        Ok(groups.into_iter().map(GroupInfo::from).collect())
    }

    /// Update group join status
    pub async fn update_join_status(&self, group_id: &str, joined: bool) -> Result<bool, DbErr> {
        debug!("Updating join status for group: {} -> {}", group_id, joined);

        let existing = groups::Entity::find()
            .filter(groups::Column::GroupId.eq(group_id))
            .one(&self.db)
            .await?;

        if let Some(model) = existing {
            let mut active_model: groups::ActiveModel = model.into();
            active_model.joined = Set(joined);
            active_model.update(&self.db).await?;
            info!("Group '{}' join status updated to {}", group_id, joined);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Update group name
    pub async fn update_name(&self, group_id: &str, name: &str) -> Result<bool, DbErr> {
        debug!("Updating name for group: {}", group_id);

        let existing = groups::Entity::find()
            .filter(groups::Column::GroupId.eq(group_id))
            .one(&self.db)
            .await?;

        if let Some(model) = existing {
            let mut active_model: groups::ActiveModel = model.into();
            active_model.name = Set(name.to_string());
            active_model.update(&self.db).await?;
            info!("Group '{}' name updated", group_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete a group
    pub async fn delete(&self, group_id: &str) -> Result<bool, DbErr> {
        debug!("Deleting group: {}", group_id);

        let result = groups::Entity::delete(groups::ActiveModel {
            group_id: Set(group_id.to_string()),
            ..Default::default()
        })
        .exec(&self.db)
        .await?;

        if result.rows_affected > 0 {
            info!("Group '{}' deleted successfully", group_id);
        }

        Ok(result.rows_affected > 0)
    }

    /// Check if a group exists
    pub async fn exists(&self, group_id: &str) -> Result<bool, DbErr> {
        let result = groups::Entity::find()
            .filter(groups::Column::GroupId.eq(group_id))
            .one(&self.db)
            .await?;

        Ok(result.is_some())
    }

    /// Check if a user has joined a group
    pub async fn is_joined(&self, group_id: &str) -> Result<bool, DbErr> {
        let result = groups::Entity::find()
            .filter(groups::Column::GroupId.eq(group_id))
            .filter(groups::Column::Joined.eq(true))
            .one(&self.db)
            .await?;

        Ok(result.is_some())
    }

    /// Clear all groups
    pub async fn clear_all(&self) -> Result<u64, DbErr> {
        info!("Clearing all groups");

        let result = groups::Entity::delete_many().exec(&self.db).await?;

        Ok(result.rows_affected)
    }

    /// Get count of all groups
    pub async fn count(&self) -> Result<u64, DbErr> {
        let count = groups::Entity::find().count(&self.db).await?;

        Ok(count)
    }

    /// Get count of joined groups
    pub async fn count_joined(&self) -> Result<u64, DbErr> {
        let count = groups::Entity::find()
            .filter(groups::Column::Joined.eq(true))
            .count(&self.db)
            .await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_info_from_model() {
        let model = groups::Model {
            group_id: "test-group".to_string(),
            name: "Test Group".to_string(),
            joined: true,
            created_at: 1234567890,
        };

        let info = GroupInfo::from(model);
        assert_eq!(info.group_id, "test-group");
        assert_eq!(info.name, "Test Group");
        assert_eq!(info.joined, true);
        assert_eq!(info.created_at, 1234567890);
    }
}
