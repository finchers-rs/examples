#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

mod api;
mod model;
mod schema;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::{Pool, PooledConnection};

use failure::Fallible;
use futures::prelude::*;
use http::StatusCode;
use serde::Deserialize;
use std::env;
use std::sync::Arc;

use finchers::prelude::*;
use finchers::rt::blocking_section;
use finchers::{output, path, routes};

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

fn main() -> Fallible<()> {
    dotenv::dotenv()?;

    let manager = ConnectionManager::<PgConnection>::new(env::var("DATABASE_URL")?);
    let pool = Pool::builder().build(manager)?;
    let acquire_conn = Arc::new(endpoint::unit().and_then(move || {
        let pool = pool.clone();
        blocking_section(move || pool.get().map_err(finchers::error::fail))
    }));

    let endpoint = path!(/"api"/"v1"/"posts").and(routes!{
        path!(@get /)
            .and(endpoints::query::optional())
            .and(acquire_conn.clone())
            .and_then(|query, conn| blocking_section(move || crate::api::get_posts(query, conn)).from_err())
            .map(output::Json),

        path!(@post /)
            .and(endpoints::body::json())
            .and(acquire_conn.clone())
            .and_then(|new_post, conn| blocking_section(move || crate::api::create_post(new_post, conn)).from_err())
            .map(output::Json)
            .map(output::status::Created),

        path!(@get / i32 /)
            .and(acquire_conn.clone())
            .and_then(|id, conn| {
                blocking_section(move ||
                crate::api::find_post(id, conn))
                    .from_err()
                    .and_then(|conn_opt| conn_opt.ok_or_else(|| finchers::error::err_msg(StatusCode::NOT_FOUND, "not found"))
                    )
            })
            .map(output::Json),
    });

    finchers::server::start(endpoint).serve("127.0.0.1:4000")?;
    Ok(())
}
