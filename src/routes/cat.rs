use axum::{
  extract::{Extension, Path},
  routing::{delete, get, post, put},
  Json, Router,
};
use bson::doc;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::context::Context;
use crate::errors::Error;
use crate::errors::NotFound;
use crate::lib::models::ModelExt;
use crate::lib::to_object_id::to_object_id;
use crate::lib::token::TokenUser;
use crate::models::cat::{Cat, PublicCat};

pub fn create_route() -> Router {
  Router::new()
    .route("/cats", post(create_cat))
    .route("/cats", get(query_cats))
    .route("/cats/:id", get(get_cat_by_id))
    .route("/cats/:id", delete(remove_cat_by_id))
    .route("/cats/:id", put(update_cat_by_id))
}

async fn create_cat(
  user: TokenUser,
  Extension(context): Extension<Context>,
  Json(payload): Json<CreateCat>,
) -> Result<Json<PublicCat>, Error> {
  let cat = Cat::new(user.id, payload.name);
  let cat = context.models.cat.create(cat).await?;
  let res = PublicCat::from(cat);

  Ok(Json(res))
}

async fn query_cats(
  user: TokenUser,
  Extension(context): Extension<Context>,
) -> Result<Json<Vec<PublicCat>>, Error> {
  let cats = context
    .models
    .cat
    .find(doc! { "user": &user.id }, None)
    .await?
    .into_iter()
    .map(Into::into)
    .collect::<Vec<PublicCat>>();

  debug!("Returning cats");
  Ok(Json(cats))
}

async fn get_cat_by_id(
  user: TokenUser,
  Extension(context): Extension<Context>,
  Path(id): Path<String>,
) -> Result<Json<PublicCat>, Error> {
  let cat_id = to_object_id(id)?;
  let cat = context
    .models
    .cat
    .find_one(doc! { "_id": cat_id, "user": &user.id }, None)
    .await?
    .map(PublicCat::from);

  let cat = match cat {
    Some(cat) => cat,
    None => {
      debug!("Cat not found, returning 404 status code");
      return Err(Error::NotFound(NotFound::new(String::from("cat"))));
    }
  };

  debug!("Returning cat");
  Ok(Json(cat))
}

async fn remove_cat_by_id(
  user: TokenUser,
  Extension(context): Extension<Context>,
  Path(id): Path<String>,
) -> Result<(), Error> {
  let cat_id = to_object_id(id)?;
  let delete_result = context
    .models
    .cat
    .delete_one(doc! { "_id": cat_id, "user": &user.id })
    .await?;

  if delete_result.deleted_count == 0 {
    debug!("Cat not found, returning 404 status code");
    return Err(Error::NotFound(NotFound::new(String::from("cat"))));
  }

  Ok(())
}

async fn update_cat_by_id(
  user: TokenUser,
  Extension(context): Extension<Context>,
  Path(id): Path<String>,
  Json(payload): Json<UpdateCat>,
) -> Result<Json<PublicCat>, Error> {
  let cat_id = to_object_id(id)?;
  let update = bson::to_document(&payload).unwrap();

  let cat = context
    .models
    .cat
    .find_one_and_update(
      doc! { "_id": &cat_id, "user": &user.id },
      doc! { "$set": update },
    )
    .await?
    .map(PublicCat::from);

  let cat = match cat {
    Some(cat) => cat,
    None => {
      debug!("Cat not found, returning 404 status code");
      return Err(Error::NotFound(NotFound::new(String::from("cat"))));
    }
  };

  debug!("Returning cat");
  Ok(Json(cat))
}

#[derive(Deserialize)]
struct CreateCat {
  name: String,
}

#[derive(Serialize, Deserialize)]
struct UpdateCat {
  name: String,
}
