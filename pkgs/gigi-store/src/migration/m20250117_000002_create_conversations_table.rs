use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum Conversations {
    Table,
    Id,
    Name,
    IsGroup,
    PeerId,
    LastMessage,
    LastMessageTime,
    LastMessageTimestamp,
    UnreadCount,
    CreatedAt,
    UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250117_000002_create_conversations_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Conversations::Table)
                    .col(
                        ColumnDef::new(Conversations::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Conversations::Name).string().not_null())
                    .col(ColumnDef::new(Conversations::IsGroup).boolean().not_null())
                    .col(ColumnDef::new(Conversations::PeerId).string().not_null())
                    .col(ColumnDef::new(Conversations::LastMessage).string())
                    .col(ColumnDef::new(Conversations::LastMessageTime).big_integer())
                    .col(ColumnDef::new(Conversations::LastMessageTimestamp).big_integer())
                    .col(
                        ColumnDef::new(Conversations::UnreadCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Conversations::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Conversations::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Conversations::Table).to_owned())
            .await
    }
}
