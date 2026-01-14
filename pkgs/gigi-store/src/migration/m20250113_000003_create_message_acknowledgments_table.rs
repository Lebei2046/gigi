use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum MessageAcknowledgments {
    Table,
    Id,
    MessageId,
    AcknowledgedByNickname,
    AcknowledgedByPeerId,
    AcknowledgedAt,
    AckType,
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    Id,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250113_000003_create_message_acknowledgments_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MessageAcknowledgments::Table)
                    .col(
                        ColumnDef::new(MessageAcknowledgments::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(MessageAcknowledgments::MessageId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MessageAcknowledgments::AcknowledgedByNickname)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MessageAcknowledgments::AcknowledgedByPeerId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MessageAcknowledgments::AcknowledgedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MessageAcknowledgments::AckType)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_acknowledgments_message")
                            .from(
                                MessageAcknowledgments::Table,
                                MessageAcknowledgments::MessageId,
                            )
                            .to(Messages::Table, Messages::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(MessageAcknowledgments::Table)
                    .to_owned(),
            )
            .await
    }
}
