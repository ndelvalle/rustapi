use axum::http::StatusCode;
use axum::{
  extract::Path,
  routing::{delete, get, post, put},
  Json, Router,
};
use bson::doc;
use serde::{Deserialize, Serialize};
use tracing::debug;
use wither::mongodb::options::FindOptions;

use crate::errors::Error;
use crate::models::cat::{Cat, PublicCat};
use crate::utils::custom_response::CustomResponseResult as Response;
use crate::utils::custom_response::{CustomResponse, CustomResponseBuilder, ResponsePagination};
use crate::utils::models::ModelExt;
use crate::utils::pagination::Pagination;
use crate::utils::to_object_id::to_object_id;
use crate::utils::token::TokenUser;

pub fn create_route() -> Router {
  Router::new()
    .route("/cats", post(create_cat))
    .route("/cats", get(query_cats))
    .route("/cats/:id", get(get_cat_by_id))
    .route("/cats/:id", delete(remove_cat_by_id))
    .route("/cats/:id", put(update_cat_by_id))
}

async fn create_cat(user: TokenUser, Json(payload): Json<CreateCat>) -> Response<PublicCat> {
  let cat = Cat::new(user.id, payload.name);
  let cat = Cat::create(cat).await?;
  let res = PublicCat::from(cat);

  let res = CustomResponseBuilder::new()
    .body(res)
    .status_code(StatusCode::CREATED)
    .build();

  Ok(res)
}

async fn query_cats(user: TokenUser, pagination: Pagination) -> Response<Vec<PublicCat>> {
  let options = FindOptions::builder()
    .sort(doc! { "created_at": -1_i32 })
    .skip(pagination.offset)
    .limit(pagination.limit as i64)
    .build();

  let (cats, count) = Cat::find_and_count(doc! { "user": &user.id }, options).await?;
  let cats = cats.into_iter().map(Into::into).collect::<Vec<PublicCat>>();

  let res = CustomResponseBuilder::new()
    .body(cats)
    .pagination(ResponsePagination {
      count,
      offset: pagination.offset,
      limit: pagination.limit,
    })
    .build();

  debug!("Returning cats");
  Ok(res)
}

async fn get_cat_by_id(user: TokenUser, Path(id): Path<String>) -> Result<Json<PublicCat>, Error> {
  let cat_id = to_object_id(id)?;
  let cat = Cat::find_one(doc! { "_id": cat_id, "user": &user.id }, None)
    .await?
    .map(PublicCat::from);

  let cat = match cat {
    Some(cat) => cat,
    None => {
      debug!("Cat not found, returning 404 status code");
      return Err(Error::not_found());
    }
  };

  debug!("Returning cat");
  Ok(Json(cat))
}

async fn remove_cat_by_id(
  user: TokenUser,
  Path(id): Path<String>,
) -> Result<CustomResponse<()>, Error> {
  let cat_id = to_object_id(id)?;
  let delete_result = Cat::delete_one(doc! { "_id": cat_id, "user": &user.id }).await?;

  if delete_result.deleted_count == 0 {
    debug!("Cat not found, returning 404 status code");
    return Err(Error::not_found());
  }

  let res = CustomResponseBuilder::new()
    .status_code(StatusCode::NO_CONTENT)
    .build();

  Ok(res)
}

async fn update_cat_by_id(
  user: TokenUser,
  Path(id): Path<String>,
  Json(payload): Json<UpdateCat>,
) -> Result<Json<PublicCat>, Error> {
  let cat_id = to_object_id(id)?;
  let update = bson::to_document(&payload).unwrap();

  let cat = Cat::find_one_and_update(
    doc! { "_id": &cat_id, "user": &user.id },
    doc! { "$set": update },
  )
  .await?
  .map(PublicCat::from);

  let cat = match cat {
    Some(cat) => cat,
    None => {
      debug!("Cat not found, returning 404 status code");
      return Err(Error::not_found());
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
