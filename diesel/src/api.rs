use diesel::prelude::*;
use failure::Fallible;
use serde::Deserialize;

use crate::model::{NewPost, Post};
use crate::schema::posts;
use crate::Conn;

#[derive(Debug, Deserialize)]
pub struct Query {
    count: i64,
}

impl Default for Query {
    fn default() -> Query {
        Query { count: 20 }
    }
}

pub fn get_posts(query: Option<Query>, conn: Conn) -> Fallible<Vec<Post>> {
    use crate::schema::posts::dsl::*;
    let query = query.unwrap_or_default();
    posts
        .limit(query.count)
        .load::<Post>(&*conn)
        .map_err(Into::into)
}

pub fn create_post(new_post: NewPost, conn: Conn) -> Fallible<Post> {
    diesel::insert_into(posts::table)
        .values(&new_post)
        .get_result::<Post>(&*conn)
        .map_err(Into::into)
}

pub fn find_post(i: i32, conn: Conn) -> Fallible<Option<Post>> {
    use crate::schema::posts::dsl::{id, posts};
    posts
        .filter(id.eq(i))
        .get_result::<Post>(&*conn)
        .optional()
        .map_err(Into::into)
}
