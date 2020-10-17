use actix_web::{web, HttpResponse};
use futures::stream::StreamExt;

use crate::errors::ApiError;
use crate::posts::model::Post;
use crate::Context;

pub fn create_router(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(get_posts)));
}

async fn get_posts(ctx: web::Data<Context>) -> actix_web::Result<HttpResponse> {
    debug!("Querying posts");
    let posts = Post::get(&ctx.database.conn)
        .await
        .map_err(|err| ApiError::WitherError(err))?
        // TODO: Collect to a Result<Vec>
        .map(|post| post.unwrap())
        .collect::<Vec<Post>>()
        .await;

    debug!("Returning posts to the client");
    let res = HttpResponse::Ok().json(posts);
    Ok(res)
}
