//! Thumbnail store - Map file paths to thumbnail paths

use anyhow::{Context, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, NotSet, QueryFilter, Set,
};
use tracing::info;

/// Thumbnail store - handles mapping of file paths to thumbnail paths
pub struct ThumbnailStore {
    db: DatabaseConnection,
}

impl ThumbnailStore {
    /// Create a new thumbnail store
    pub async fn new(db: DatabaseConnection) -> Result<Self> {
        Ok(Self { db })
    }

    /// Store or update a thumbnail mapping
    pub async fn store_thumbnail(&self, file_path: &str, thumbnail_path: &str) -> Result<()> {
        use crate::entities::thumbnails;
        use sea_orm::DbErr;

        let created_at = chrono::Utc::now().timestamp();

        // Check if mapping already exists
        let existing = thumbnails::Entity::find()
            .filter(thumbnails::Column::FilePath.eq(file_path))
            .one(&self.db)
            .await
            .context("Failed to query existing thumbnail mapping")?;

        if let Some(existing) = existing {
            // Update existing
            let mut active_model: thumbnails::ActiveModel = existing.into();
            active_model.thumbnail_path = Set(thumbnail_path.to_string());
            active_model
                .update(&self.db)
                .await
                .context("Failed to update thumbnail mapping")?;
        } else {
            // Insert new
            let new_mapping = thumbnails::ActiveModel {
                id: NotSet,
                file_path: Set(file_path.to_string()),
                thumbnail_path: Set(thumbnail_path.to_string()),
                created_at: Set(created_at),
            };
            // Ignore RecordNotFound error - insert likely succeeded
            match new_mapping.insert(&self.db).await {
                Ok(_) | Err(DbErr::RecordNotFound(_)) => {}
                Err(e) => return Err(e).context("Failed to insert thumbnail mapping")?,
            }
        }

        info!(
            "Stored thumbnail mapping: {} -> {}",
            file_path, thumbnail_path
        );
        Ok(())
    }

    /// Get thumbnail path by file path
    pub async fn get_thumbnail(&self, file_path: &str) -> Result<Option<String>> {
        use crate::entities::thumbnails;

        let result = thumbnails::Entity::find()
            .filter(thumbnails::Column::FilePath.eq(file_path))
            .one(&self.db)
            .await
            .context("Failed to query thumbnail mapping")?;

        Ok(result.map(|r| r.thumbnail_path))
    }

    /// Delete thumbnail mapping by file path
    pub async fn delete_thumbnail(&self, file_path: &str) -> Result<bool> {
        use crate::entities::thumbnails;

        let result = thumbnails::Entity::delete_many()
            .filter(thumbnails::Column::FilePath.eq(file_path))
            .exec(&self.db)
            .await
            .context("Failed to delete thumbnail mapping")?;

        info!("Deleted thumbnail mapping for: {}", file_path);
        Ok(result.rows_affected > 0)
    }

    /// Clean up old thumbnails (older than specified seconds)
    pub async fn cleanup_old_thumbnails(&self, older_than_seconds: i64) -> Result<u64> {
        use crate::entities::thumbnails;

        let cutoff = chrono::Utc::now().timestamp() - older_than_seconds;

        let result = thumbnails::Entity::delete_many()
            .filter(thumbnails::Column::CreatedAt.lt(cutoff))
            .exec(&self.db)
            .await
            .context("Failed to cleanup old thumbnails")?;

        info!("Cleaned up {} old thumbnail mappings", result.rows_affected);
        Ok(result.rows_affected)
    }
}
