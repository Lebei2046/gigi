use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum OfflineQueue {
    Table,
    MessageId,
    TargetNickname,
    TargetPeerId,
    QueuedAt,
    RetryCount,
    MaxRetries,
    LastRetryAt,
    NextRetryAt,
    ExpiresAt,
    Status,
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    Id,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250113_000002_create_offline_queue_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OfflineQueue::Table)
                    .col(
                        ColumnDef::new(OfflineQueue::MessageId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OfflineQueue::TargetNickname)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(OfflineQueue::TargetPeerId).string())
                    .col(
                        ColumnDef::new(OfflineQueue::QueuedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OfflineQueue::RetryCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(OfflineQueue::MaxRetries)
                            .integer()
                            .not_null()
                            .default(10),
                    )
                    .col(ColumnDef::new(OfflineQueue::LastRetryAt).big_integer())
                    .col(ColumnDef::new(OfflineQueue::NextRetryAt).big_integer())
                    .col(
                        ColumnDef::new(OfflineQueue::ExpiresAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OfflineQueue::Status)
                            .string()
                            .not_null()
                            .default("Pending"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_offline_queue_message")
                            .from(OfflineQueue::Table, OfflineQueue::MessageId)
                            .to(Messages::Table, Messages::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OfflineQueue::Table).to_owned())
            .await
    }
}
