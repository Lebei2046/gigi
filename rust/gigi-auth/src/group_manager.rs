use sea_orm::DatabaseConnection;

pub use crate::settings_manager::GroupInfo;

pub struct GroupManager {
    db: DatabaseConnection,
}

impl GroupManager {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn add_or_update(&self, group_id: &str, name: &str, joined: bool) -> Result<(), sea_orm::DbErr> {
        crate::settings_manager::SettingsManager::new(self.db.clone())
            .upsert_group(group_id, name, joined)
            .await
    }

    pub async fn get(&self, group_id: &str) -> Result<Option<GroupInfo>, sea_orm::DbErr> {
        crate::settings_manager::SettingsManager::new(self.db.clone())
            .get_group(group_id)
            .await
    }

    pub async fn get_all(&self) -> Result<Vec<GroupInfo>, sea_orm::DbErr> {
        crate::settings_manager::SettingsManager::new(self.db.clone())
            .get_all_groups()
            .await
    }

    pub async fn delete(&self, group_id: &str) -> Result<bool, sea_orm::DbErr> {
        crate::settings_manager::SettingsManager::new(self.db.clone())
            .delete_group(group_id)
            .await
    }

    pub async fn update_name(&self, group_id: &str, name: &str) -> Result<bool, sea_orm::DbErr> {
        crate::settings_manager::SettingsManager::new(self.db.clone())
            .update_group_name(group_id, name)
            .await
    }

    pub async fn update_join_status(&self, group_id: &str, joined: bool) -> Result<bool, sea_orm::DbErr> {
        crate::settings_manager::SettingsManager::new(self.db.clone())
            .update_group_join_status(group_id, joined)
            .await
    }
}