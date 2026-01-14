use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum AppData {
    Table,
    Key,
    Nickname,
    CreatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250113_000004_create_app_data_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AppData::Table)
                    .col(
                        ColumnDef::new(AppData::Key)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AppData::Nickname).string())
                    .col(ColumnDef::new(AppData::CreatedAt).big_integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AppData::Table).to_owned())
            .await
    }
}
