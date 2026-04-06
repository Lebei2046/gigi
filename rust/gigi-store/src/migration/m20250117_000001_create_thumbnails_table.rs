use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Thumbnails::Table)
                    .if_not_exists()
                    .col(pk_auto(Thumbnails::Id))
                    .col(string_uniq(Thumbnails::FilePath))
                    .col(string(Thumbnails::ThumbnailPath))
                    .col(integer(Thumbnails::CreatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Thumbnails::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Thumbnails {
    Table,
    Id,
    FilePath,
    ThumbnailPath,
    CreatedAt,
}
