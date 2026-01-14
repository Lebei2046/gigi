//! Sea-ORM migrations for gigi-store database schema

pub use sea_orm_migration::prelude::*;

mod m20250113_000001_create_messages_table;
mod m20250113_000002_create_offline_queue_table;
mod m20250113_000003_create_message_acknowledgments_table;
mod m20250113_000004_create_app_data_table;
mod m20250114_000001_create_shared_files_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250113_000001_create_messages_table::Migration),
            Box::new(m20250113_000002_create_offline_queue_table::Migration),
            Box::new(m20250113_000003_create_message_acknowledgments_table::Migration),
            Box::new(m20250113_000004_create_app_data_table::Migration),
            Box::new(m20250114_000001_create_shared_files_table::Migration),
        ]
    }
}
