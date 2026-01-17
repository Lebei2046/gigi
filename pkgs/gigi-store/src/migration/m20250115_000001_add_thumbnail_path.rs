use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum SharedFiles {
    Table,
    ThumbnailPath,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250115_000001_add_thumbnail_path"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add thumbnail_path column to existing shared_files table
        manager
            .alter_table(
                Table::alter()
                    .table(SharedFiles::Table)
                    .add_column(ColumnDef::new(SharedFiles::ThumbnailPath).string().null())
                    .to_owned(),
            )
            .await?;

        // Create index for thumbnail_path for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_shared_files_thumbnail")
                    .table(SharedFiles::Table)
                    .col(SharedFiles::ThumbnailPath)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index
        manager
            .drop_index(Index::drop().name("idx_shared_files_thumbnail").to_owned())
            .await?;

        // Remove thumbnail_path column
        manager
            .alter_table(
                Table::alter()
                    .table(SharedFiles::Table)
                    .drop_column(SharedFiles::ThumbnailPath)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
