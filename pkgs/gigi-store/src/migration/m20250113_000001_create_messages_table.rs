use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum Messages {
    Table,
    Id,
    MsgType,
    Direction,
    ContentType,
    ContentJson,
    SenderNickname,
    RecipientNickname,
    GroupName,
    PeerId,
    Timestamp,
    CreatedAt,
    Delivered,
    DeliveredAt,
    Read,
    ReadAt,
    SyncStatus,
    SyncAttempts,
    LastSyncAttempt,
    ExpiresAt,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250113_000001_create_messages_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Messages::Table)
                    .col(
                        ColumnDef::new(Messages::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Messages::MsgType).string().not_null())
                    .col(ColumnDef::new(Messages::Direction).string().not_null())
                    .col(ColumnDef::new(Messages::ContentType).string().not_null())
                    .col(ColumnDef::new(Messages::ContentJson).string().not_null())
                    .col(ColumnDef::new(Messages::SenderNickname).string().not_null())
                    .col(ColumnDef::new(Messages::RecipientNickname).string())
                    .col(ColumnDef::new(Messages::GroupName).string())
                    .col(ColumnDef::new(Messages::PeerId).string().not_null())
                    .col(ColumnDef::new(Messages::Timestamp).big_integer().not_null())
                    .col(ColumnDef::new(Messages::CreatedAt).big_integer().not_null())
                    .col(
                        ColumnDef::new(Messages::Delivered)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Messages::DeliveredAt).big_integer())
                    .col(
                        ColumnDef::new(Messages::Read)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Messages::ReadAt).big_integer())
                    .col(
                        ColumnDef::new(Messages::SyncStatus)
                            .string()
                            .not_null()
                            .default("Pending"),
                    )
                    .col(
                        ColumnDef::new(Messages::SyncAttempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Messages::LastSyncAttempt).big_integer())
                    .col(ColumnDef::new(Messages::ExpiresAt).big_integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Messages::Table).to_owned())
            .await
    }
}
