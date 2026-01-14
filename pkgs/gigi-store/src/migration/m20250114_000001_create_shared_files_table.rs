use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum SharedFiles {
    Table,
    ShareCode,
    FileName,
    FilePath,
    FileSize,
    Hash,
    ChunkCount,
    CreatedAt,
    Revoked,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250114_000001_create_shared_files_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SharedFiles::Table)
                    .col(
                        ColumnDef::new(SharedFiles::ShareCode)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SharedFiles::FileName).string().not_null())
                    .col(ColumnDef::new(SharedFiles::FilePath).string().not_null())
                    .col(
                        ColumnDef::new(SharedFiles::FileSize)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SharedFiles::Hash).string().not_null())
                    .col(ColumnDef::new(SharedFiles::ChunkCount).integer().not_null())
                    .col(
                        ColumnDef::new(SharedFiles::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SharedFiles::Revoked).boolean().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SharedFiles::Table).to_owned())
            .await
    }
}
