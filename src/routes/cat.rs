use axum::{
  extract::{Extension, Path},
  routing::{get, post},
  Json, Router,
};
use bson::doc;
use serde::Deserialize;
use tracing::debug;

use crate::context::Context;
use crate::errors::Error;
use crate::errors::NotFound;
use crate::lib::to_object_id::to_object_id;
use crate::lib::token::TokenUser;
use crate::models::cat::{Cat, PublicCat};
use crate::models::ModelExt;

pub fn create_route() -> Router {
  Router::new()
    .route("/cats", post(create_cat))
    .route("/cats", get(query_cats))
    .route("/cats/:id", get(get_cat_by_id))
}

#[derive(Deserialize)]
struct CreateCat {
  name: String,
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
