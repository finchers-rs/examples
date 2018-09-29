use diesel::prelude::*;
use failure::Error;
use futures::prelude::*;
use serde::Deserialize;

use crate::model::{NewPost, Post};
use crate::schema::posts;
use crate::util::execute_blocking_code;
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

pub fn get_posts(query: Option<Query>, conn: Conn) -> impl Future<Item = Vec<Post>, Error = Error> {
    let query = query.unwrap_or_default();
    execute_blocking_code(move || {
        use crate::schema::posts::dsl::*;
        posts.limit(query.count).load::<Post>(&*conn)
    })
}

pub fn create_post(new_post: NewPost, conn: Conn) -> impl Future<Item = Post, Error = Error> {
    execute_blocking_code(move || {
        diesel::insert_into(posts::table)
            .values(&new_post)
            .get_result::<Post>(&*conn)
    })
}

pub fn find_post(i: i32, conn: Conn) -> impl Future<Item = Option<Post>, Error = Error> {
    execute_blocking_code(move || {
        use crate::schema::posts::dsl::{id, posts};
        posts.filter(id.eq(i)).get_result::<Post>(&*conn).optional()
    })
}
