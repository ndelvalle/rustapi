use crate::components::posts::model::{NewPost, Post};
use crate::database::Connection as DBConnection;
use rocket::http::Status;
use rocket::State;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get("/posts", format = "application/json")]
pub fn find(logger: State<slog::Logger>, db: DBConnection) -> Json<Value> {
  debug!(logger, "Querying posts");

  Json(json!(Post::find(&db)))
}

#[post("/posts", format = "application/json", data = "<new_post>")]
pub fn create(new_post: Json<NewPost>, logger: State<slog::Logger>, db: DBConnection) -> Status {
  debug!(logger, "Creating new post");

  Post::create(new_post.into_inner(), &db);
  Status::Created
}

#[get("/posts/<id>", format = "application/json")]
pub fn find_by_id(id: i32, logger: State<slog::Logger>, db: DBConnection) -> Json<Value> {
  debug!(logger, "Querying posts by ID {}", id);

  Json(json!(Post::find_by_id(id, &db)))
}

#[put("/posts/<id>", format = "application/json", data = "<post>")]
pub fn update(
  id: i32,
  post: Json<NewPost>,
  logger: State<slog::Logger>,
  db: DBConnection,
) -> Status {
  debug!(logger, "Updating post with ID {}", id);

  Post::update(id, &db, post.into_inner());

  Status::Ok
}

#[delete("/posts/<id>")]
pub fn delete(id: i32, logger: State<slog::Logger>, db: DBConnection) -> Status {
  debug!(logger, "Removing post with ID {}", id);

  Post::delete(id, &db);

  Status::Ok
}

#[get("/posts/title/<title>", format = "application/json")]
pub fn find_by_title(title: String, logger: State<slog::Logger>, db: DBConnection) -> Json<Value> {
  debug!(logger, "Querying post with title {}", &title);

  Json(json!(Post::find_by_title(title, &db)))
}
