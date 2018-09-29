use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use failure::Error;
use futures::prelude::*;
use tokio_threadpool::blocking;

pub struct Connection {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl Connection {
    pub fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Connection { conn }
    }

    pub fn get(&self) -> &PgConnection {
        &*self.conn
    }

    pub fn execute<F, T, E>(self, f: F) -> impl Future<Item = T, Error = Error>
    where
        F: FnOnce(&Connection) -> Result<T, E>,
        E: Into<Error>,
    {
        let mut f_opt = Some(f);
        futures::future::poll_fn(move || {
            let x = futures::try_ready!(blocking(|| (f_opt.take().unwrap())(&self)))
                .map_err(Into::into)?;
            Ok(x.into())
        })
    }
}
