use crate::database;
use crate::errors::Error;
use async_trait::async_trait;

use sea_orm::query::QueryFilter;
use sea_orm::{ActiveModelTrait, DeleteResult, EntityTrait, PrimaryKeyTrait};

#[async_trait]
pub trait EntityExt: EntityTrait {
    // async fn create<T>(model: T) -> Result<(), Error>
    // where
    //     T: ActiveModelTrait<Entity = Self> + Send,
    // {
    //     let conn = database::connection().await;
    //     // let result = <Self as EntityTrait>::insert(model)
    //     //     .exec(conn)
    //     //     .await
    //     //     .map_err(Error::DatabaseError)?;
    //
    //     Self::ActiveModel {
    //         title: Set(form_data.title.to_owned()),
    //         text: Set(form_data.text.to_owned()),
    //         ..Default::default()
    //     }
    //     .save(db)
    //     .await
    // }

    async fn create<T>(model: T) -> Result<Self, Error>
    where
        T: ActiveModelTrait<Entity = Self> + Send,
    {
        let conn = database::connection().await;
        model.save(conn).await
    }

    async fn find_by_id<T>(id: T) -> Result<Option<Self::Model>, Error>
    where
        T: Into<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send,
    {
        let conn = database::connection().await;
        <Self as EntityTrait>::find_by_id(id)
            .one(conn)
            .await
            .map_err(Error::DatabaseError)
    }

    async fn find_one<Q>(query: Q) -> Result<Option<Self::Model>, Error>
    where
        Q: QueryFilter,
    {
        let conn = database::connection().await;
        <Self as EntityTrait>::find()
            .filter(query)
            .one(conn)
            .await
            .map_err(Error::DatabaseError)
    }

    async fn find<Q>(query: Q) -> Result<Option<Self::Model>, Error>
    where
        Q: QueryFilter,
    {
        let conn = database::connection().await;
        <Self as EntityTrait>::find()
            .filter(query)
            .all(conn)
            .await
            .map_err(Error::DatabaseError)
    }

    async fn delete_by_id<T>(id: T) -> Result<DeleteResult, Error>
    where
        T: Into<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send,
    {
        let conn = database::connection().await;
        <Self as EntityTrait>::delete_by_id(id)
            .exec(conn)
            .await
            .map_err(Error::DatabaseError)
    }

    async fn update_one<Q, U>(query: Q, update: U) -> Result<DeleteResult, Error>
    where
        Q: QueryFilter,
        U: ActiveModelTrait<Entity = Self>,
    {
        let conn = database::connection().await;
        <Self as EntityTrait>::update(update.clone())
            .filter(query)
            .exec(conn)
            .await
            .map_err(Error::DatabaseError)
    }
}
