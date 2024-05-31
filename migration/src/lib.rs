pub use sea_orm_migration::prelude::*;

mod m20240527_080925_create_users_table;
mod m20240530_082146_create_cats_table;

use m20240527_080925_create_users_table as users;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m20240527_080925_create_users_table::Migration),
      Box::new(m20240530_082146_create_cats_table::Migration),
    ]
  }
}
