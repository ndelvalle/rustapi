use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;

use super::schema::posts;
use super::schema::posts::dsl::posts as collection;

#[derive(Serialize, Queryable, Debug, Clone)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "posts"]
pub struct NewPost {
    pub title: String,
    pub body: String,
    pub published: bool,
}

impl Post {
    pub fn find(conn: &PgConnection) -> Vec<Post> {
        collection
            .order(posts::id.desc())
            .load::<Post>(&*conn)
            .expect("error loading posts")
    }

    pub fn create(post: NewPost, conn: &PgConnection) -> bool {
        diesel::insert_into(posts::table)
            .values(&post)
            .execute(conn)
            .is_ok()
    }

    pub fn find_by_id(id: i32, conn: &PgConnection) -> Vec<Post> {
        collection
            .find(id)
            .load::<Post>(conn)
            .expect("Error loading post")
    }

    pub fn update(id: i32, conn: &PgConnection, post: NewPost) -> bool {
        use super::schema::posts::dsl::{body as b, published as p, title as t};
        let NewPost {
            title,
            body,
            published,
        } = post;

        diesel::update(collection.find(id))
            .set((b.eq(body), p.eq(published), t.eq(title)))
            .get_result::<Post>(conn)
            .is_ok()
    }

    pub fn delete(id: i32, conn: &PgConnection) -> bool {
        if Post::find_by_id(id, conn).is_empty() {
            return false;
        };
        diesel::delete(collection.find(id)).execute(conn).is_ok()
    }

    pub fn find_by_title(title: String, conn: &PgConnection) -> Vec<Post> {
        collection
            .filter(posts::title.eq(title))
            .load::<Post>(conn)
            .expect("Error loading posts by title")
    }
}
