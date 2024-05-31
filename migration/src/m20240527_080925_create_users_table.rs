use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum User {
  Table,
  Id,
  Name,
  Email,
  Password,
  CreatedAt,
  UpdatedAt,
  LockedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(User::Table)
          .if_not_exists()
          .col(ColumnDef::new(User::Id).uuid().not_null().primary_key())
          .col(ColumnDef::new(User::Name).string().not_null())
          .col(ColumnDef::new(User::Email).string().not_null())
          .col(ColumnDef::new(User::Password).string().not_null())
          .col(
            ColumnDef::new(User::CreatedAt)
              .timestamp()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .col(
            ColumnDef::new(User::UpdatedAt)
              .timestamp()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .col(ColumnDef::new(User::LockedAt).timestamp())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(User::Table).to_owned())
      .await
  }
}
