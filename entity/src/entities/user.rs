//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub name: String,
  pub email: String,
  pub password: String,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  pub locked_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::cat::Entity")]
  Cat,
}

impl Related<super::cat::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Cat.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
