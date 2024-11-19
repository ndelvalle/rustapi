use async_trait::async_trait;

use diesel::pg::Pg;

use diesel::associations::HasTable;
use diesel::prelude::Queryable;
use diesel::prelude::Table;
use diesel_async::RunQueryDsl;

use crate::database;
use crate::errors::Error;

#[async_trait]
pub trait QueryModelExt<T>
where
    Self: Queryable<T, Pg> + HasTable<Table = T> + Send + Sync,
    T: Table + Send + Sync,
{
    async fn find_by_id(id: i32) -> Result<Option<Self>, Error> {
        let conn = database::get_connection().await;
        use diesel::prelude::*;
        Self::table().find(id).first::<Self>(conn).optional()
    }
}

impl<T, U> QueryModelExt<T> for U
where
    U: Queryable<T, Pg> + HasTable<Table = T> + Send + Sync,
    T: Table + Send + Sync,
{
}

// #[async_trait]
// pub trait QueryModelExt: Sized {
//     type Table: Table;
//
//     async fn find_by_id(id: i32) -> Result<Option<Self>, Error>
//     where
//         Self: Queryable<Self::Table::SqlType, Pg>,
//         Self::Table: FindDsl<i32>,
//     {
//         let mut conn = database::get_connection().await;
//         Self::Table::table()
//             .find(id)
//             .first::<Self>(&mut conn)
//             .await
//             .optional()
//             .map_err(Error::Diesel)
//     }
// }
//
// impl<T> QueryModelExt for T
// where
//     T: HasTable + Queryable<T::Table::SqlType, Pg> + Send + Sync,
//     T::Table: Table + FindDsl<i32> + Send + Sync,
// {
//     type Table = T::Table;
// }
