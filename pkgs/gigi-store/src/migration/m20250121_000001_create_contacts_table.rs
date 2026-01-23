use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum Contacts {
    Table,
    PeerId,
    Name,
    AddedAt,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250121_000001_create_contacts_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Contacts::Table)
                    .col(
                        ColumnDef::new(Contacts::PeerId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Contacts::Name).string().not_null())
                    .col(ColumnDef::new(Contacts::AddedAt).big_integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Contacts::Table).to_owned())
            .await
    }
}
