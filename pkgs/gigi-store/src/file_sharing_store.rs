//! File sharing store - Store and retrieve shared file information

use anyhow::{Context, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Shared file information stored in gigi-store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFileInfo {
    pub share_code: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub hash: String,
    pub chunk_count: usize,
    pub thumbnail_path: Option<String>,
    pub created_at: i64,
    pub revoked: bool,
}

impl SharedFileInfo {
    /// Create from file metadata
    pub fn new(
        share_code: String,
        file_name: String,
        file_path: String,
        file_size: u64,
        hash: String,
        chunk_count: usize,
        created_at: i64,
    ) -> Self {
        Self {
            share_code,
            file_name,
            file_path,
            file_size,
            hash,
            chunk_count,
            thumbnail_path: None,
            created_at,
            revoked: false,
        }
    }
}

/// File sharing store - handles storage and retrieval of shared file information
pub struct FileSharingStore {
    db: DatabaseConnection,
}

impl FileSharingStore {
    /// Create a new file sharing store
    pub async fn new(db: DatabaseConnection) -> Result<Self> {
        Ok(Self { db })
    }

    /// Store or update a shared file
    pub async fn store_shared_file(&self, info: &SharedFileInfo) -> Result<()> {
        use crate::entities::shared_files;
        use sea_orm::DbErr;

        // Check if file already exists
        let existing = shared_files::Entity::find()
            .filter(shared_files::Column::ShareCode.eq(&info.share_code))
            .one(&self.db)
            .await
            .context("Failed to query existing shared file")?;

        if let Some(existing) = existing {
            // Update existing
            let mut active_model: shared_files::ActiveModel = existing.into();
            active_model.file_name = Set(info.file_name.clone());
            active_model.file_path = Set(info.file_path.clone());
            active_model.file_size = Set(info.file_size as i64);
            active_model.hash = Set(info.hash.clone());
            active_model.chunk_count = Set(info.chunk_count as i32);
            active_model.thumbnail_path = Set(info.thumbnail_path.clone());
            active_model.revoked = Set(info.revoked);
            active_model
                .update(&self.db)
                .await
                .context("Failed to update shared file")?;
        } else {
            // Insert new
            let new_file = shared_files::ActiveModel {
                share_code: Set(info.share_code.clone()),
                file_name: Set(info.file_name.clone()),
                file_path: Set(info.file_path.clone()),
                file_size: Set(info.file_size as i64),
                hash: Set(info.hash.clone()),
                chunk_count: Set(info.chunk_count as i32),
                thumbnail_path: Set(info.thumbnail_path.clone()),
                created_at: Set(info.created_at),
                revoked: Set(info.revoked),
            };
            // Ignore RecordNotFound error - insert likely succeeded
            match new_file.insert(&self.db).await {
                Ok(_) | Err(DbErr::RecordNotFound(_)) => {}
                Err(e) => return Err(e).context("Failed to insert shared file")?,
            }
        }

        info!(
            "Stored shared file: {} ({})",
            info.file_name, info.share_code
        );
        Ok(())
    }

    /// Retrieve a shared file by share code
    pub async fn get_shared_file(&self, share_code: &str) -> Result<Option<SharedFileInfo>> {
        use crate::entities::shared_files;

        let result = shared_files::Entity::find()
            .filter(shared_files::Column::ShareCode.eq(share_code))
            .one(&self.db)
            .await
            .context("Failed to query shared file")?;

        Ok(result.map(|data| SharedFileInfo {
            share_code: data.share_code,
            file_name: data.file_name,
            file_path: data.file_path,
            file_size: data.file_size as u64,
            hash: data.hash,
            chunk_count: data.chunk_count as usize,
            thumbnail_path: data.thumbnail_path,
            created_at: data.created_at,
            revoked: data.revoked,
        }))
    }

    /// List all shared files
    pub async fn list_shared_files(&self) -> Result<Vec<SharedFileInfo>> {
        use crate::entities::shared_files;

        let results = shared_files::Entity::find()
            .all(&self.db)
            .await
            .context("Failed to list shared files")?;

        Ok(results
            .into_iter()
            .map(|data| SharedFileInfo {
                share_code: data.share_code,
                file_name: data.file_name,
                file_path: data.file_path,
                file_size: data.file_size as u64,
                hash: data.hash,
                chunk_count: data.chunk_count as usize,
                thumbnail_path: data.thumbnail_path,
                created_at: data.created_at,
                revoked: data.revoked,
            })
            .collect())
    }

    /// Delete (unshare) a file by share code
    pub async fn delete_shared_file(&self, share_code: &str) -> Result<bool> {
        use crate::entities::shared_files;

        let result = shared_files::Entity::delete_many()
            .filter(shared_files::Column::ShareCode.eq(share_code))
            .exec(&self.db)
            .await
            .context("Failed to delete shared file")?;

        info!("Deleted shared file with code: {}", share_code);
        Ok(result.rows_affected > 0)
    }

    /// Mark a file as revoked
    pub async fn revoke_shared_file(&self, share_code: &str) -> Result<bool> {
        use crate::entities::shared_files;

        let existing = shared_files::Entity::find()
            .filter(shared_files::Column::ShareCode.eq(share_code))
            .one(&self.db)
            .await
            .context("Failed to query shared file")?;

        if let Some(existing) = existing {
            let mut active_model: shared_files::ActiveModel = existing.into();
            active_model.revoked = Set(true);
            active_model
                .update(&self.db)
                .await
                .context("Failed to revoke shared file")?;
            info!("Revoked shared file with code: {}", share_code);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Clean up revoked files
    pub async fn cleanup_revoked_files(&self) -> Result<u64> {
        use crate::entities::shared_files;

        let result = shared_files::Entity::delete_many()
            .filter(shared_files::Column::Revoked.eq(true))
            .exec(&self.db)
            .await
            .context("Failed to cleanup revoked files")?;

        info!("Cleaned up {} revoked shared files", result.rows_affected);
        Ok(result.rows_affected)
    }

    /// Update thumbnail path for a file
    pub async fn update_thumbnail_path(
        &self,
        share_code: &str,
        thumbnail_path: &str,
    ) -> Result<()> {
        use crate::entities::shared_files;

        let existing = shared_files::Entity::find()
            .filter(shared_files::Column::ShareCode.eq(share_code))
            .one(&self.db)
            .await
            .context("Failed to query shared file")?;

        if let Some(existing) = existing {
            let mut active_model: shared_files::ActiveModel = existing.into();
            active_model.thumbnail_path = Set(Some(thumbnail_path.to_string()));
            active_model
                .update(&self.db)
                .await
                .context("Failed to update thumbnail path")?;
            info!("Updated thumbnail path for: {}", share_code);
        }

        Ok(())
    }

    /// Get thumbnail path by share code
    pub async fn get_thumbnail_path(&self, share_code: &str) -> Result<Option<String>> {
        use crate::entities::shared_files;

        let result = shared_files::Entity::find()
            .filter(shared_files::Column::ShareCode.eq(share_code))
            .one(&self.db)
            .await
            .context("Failed to query shared file")?;

        Ok(result.and_then(|r| r.thumbnail_path))
    }
}
