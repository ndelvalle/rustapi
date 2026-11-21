use axum::http::StatusCode;
use axum::{
    extract::Path,
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;
use validator::Validate;

use crate::errors::Error;
use crate::models::cat::Cat;
use crate::utils::authenticate_request::authenticate_request;
use crate::utils::custom_response::CustomResponseResult as Response;
use crate::utils::custom_response::{CustomResponse, CustomResponseBuilder, ResponsePagination};
use crate::utils::date::Date;
use crate::utils::pagination::Pagination;
use crate::utils::to_uuid::to_uuid;
use crate::utils::token::TokenUser;

pub fn create_route() -> Router {
    Router::new()
        .route("/cats", post(create_cat))
        .route("/cats", get(query_cats))
        .route("/cats/{id}", get(get_cat_by_id))
        .route("/cats/{id}", delete(remove_cat_by_id))
        .route("/cats/{id}", put(update_cat_by_id))
        .route_layer(middleware::from_fn(authenticate_request))
}

async fn create_cat(user: TokenUser, Json(payload): Json<CreateCat>) -> Response<CatResponse> {
    payload.validate().map_err(|_| Error::bad_request())?;

    let cat = Cat::new(user.id, payload.name);
    let cat = Cat::create(cat).await?;
    let res = CatResponse::from(cat);

    let res = CustomResponseBuilder::new()
        .body(res)
        .status_code(StatusCode::CREATED)
        .build();

    Ok(res)
}

async fn query_cats(user: TokenUser, pagination: Pagination) -> Response<Vec<CatResponse>> {
    let count = Cat::count_by_user(user.id).await?;
    let cats = Cat::find_by_user(user.id, pagination.limit, pagination.offset).await?;
    let cats = cats
        .into_iter()
        .map(Into::into)
        .collect::<Vec<CatResponse>>();

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

async fn get_cat_by_id(
    user: TokenUser,
    Path(id): Path<String>,
) -> Result<Json<CatResponse>, Error> {
    let cat_id = to_uuid(id)?;
    let cat = Cat::find_by_id(user.id, cat_id)
        .await?
        .map(CatResponse::from);

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
    let cat_id = to_uuid(id)?;
    let deleted = Cat::delete(user.id, cat_id).await?;

    if !deleted {
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
) -> Result<Json<CatResponse>, Error> {
    payload.validate().map_err(|_| Error::bad_request())?;

    let cat_id = to_uuid(id)?;
    let cat = Cat::update_name(user.id, cat_id, &payload.name)
        .await?
        .map(CatResponse::from);

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

#[derive(Deserialize, Validate)]
struct CreateCat {
    #[validate(length(min = 1))]
    name: String,
}

#[derive(Deserialize, Validate)]
struct UpdateCat {
    #[validate(length(min = 1))]
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatResponse {
    pub id: Uuid,
    pub user: Uuid,
    pub name: String,
    pub updated_at: Date,
    pub created_at: Date,
}

impl From<Cat> for CatResponse {
    fn from(cat: Cat) -> Self {
        Self {
            id: cat.id,
            user: cat.user_id,
            name: cat.name,
            updated_at: cat.updated_at,
            created_at: cat.created_at,
        }
    }
}
