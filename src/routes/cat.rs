use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;
use validator::Validate;

use crate::app::Context;
use crate::common::authenticate_request::{authenticate_request, AuthenticatedUser};
use crate::common::custom_response::CustomResponseResult as Response;
use crate::common::custom_response::{CustomResponse, CustomResponseBuilder, ResponsePagination};
use crate::common::date::Date;
use crate::common::pagination::Pagination;
use crate::common::to_uuid::to_uuid;
use crate::errors::Error;
use crate::models::cat::Cat;

pub fn create_route(ctx: Context) -> Router<Context> {
    Router::new()
        .route("/cats", post(create_cat))
        .route("/cats", get(query_cats))
        .route("/cats/{id}", get(get_cat_by_id))
        .route("/cats/{id}", delete(remove_cat_by_id))
        .route("/cats/{id}", put(update_cat_by_id))
        .route_layer(middleware::from_fn_with_state(ctx, authenticate_request))
}

async fn create_cat(
    State(ctx): State<Context>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateCat>,
) -> Response<CatResponse> {
    payload.validate()?;

    let cat = Cat::create(&ctx.db, user.user_id, &payload.name).await?;
    let response = CatResponse::from(cat);

    let response = CustomResponseBuilder::new()
        .body(response)
        .status_code(StatusCode::CREATED)
        .build();

    Ok(response)
}

async fn query_cats(
    State(ctx): State<Context>,
    user: AuthenticatedUser,
    pagination: Pagination,
) -> Response<Vec<CatResponse>> {
    let count = Cat::count_by_user(&ctx.db, user.user_id).await?;
    let cats =
        Cat::find_by_user(&ctx.db, user.user_id, pagination.limit, pagination.offset).await?;
    let cats = cats
        .into_iter()
        .map(Into::into)
        .collect::<Vec<CatResponse>>();

    let response = CustomResponseBuilder::new()
        .body(cats)
        .pagination(ResponsePagination {
            count,
            offset: pagination.offset,
            limit: pagination.limit,
        })
        .build();

    debug!("Returning cats");
    Ok(response)
}

async fn get_cat_by_id(
    State(ctx): State<Context>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<CatResponse>, Error> {
    let cat_id = to_uuid(id)?;
    let cat = Cat::find_by_id(&ctx.db, user.user_id, cat_id)
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
    State(ctx): State<Context>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<CustomResponse<()>, Error> {
    let cat_id = to_uuid(id)?;
    let deleted = Cat::delete(&ctx.db, user.user_id, cat_id).await?;

    if !deleted {
        debug!("Cat not found, returning 404 status code");
        return Err(Error::not_found());
    }

    let response = CustomResponseBuilder::new()
        .status_code(StatusCode::NO_CONTENT)
        .build();

    Ok(response)
}

async fn update_cat_by_id(
    State(ctx): State<Context>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCat>,
) -> Result<Json<CatResponse>, Error> {
    payload.validate()?;

    let cat_id = to_uuid(id)?;
    let cat = Cat::update_name(&ctx.db, user.user_id, cat_id, &payload.name)
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
    #[validate(length(min = 1, message = "must not be empty"))]
    name: String,
}

#[derive(Deserialize, Validate)]
struct UpdateCat {
    #[validate(length(min = 1, message = "must not be empty"))]
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
            id: cat.cat_id,
            user: cat.user_id,
            name: cat.name,
            updated_at: cat.updated_at,
            created_at: cat.created_at,
        }
    }
}
