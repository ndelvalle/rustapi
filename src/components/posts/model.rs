use serde::{Deserialize, Serialize};
use wither::bson::{doc, oid::ObjectId};
use wither::mongodb;
use wither::ModelCursor;
use wither::{prelude::*, Result};

#[derive(Debug, Model, Serialize, Deserialize)]
pub struct Post {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub body: String,
    pub is_published: bool,
}

impl Post {
    pub async fn get(conn: &mongodb::Database) -> Result<ModelCursor<Post>> {
        let cursor = Post::find(conn, None, None).await?;
        Ok(cursor)
    }
}
