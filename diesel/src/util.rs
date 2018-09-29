use failure::Error;
use futures::future::poll_fn;
use futures::prelude::*;
use tokio_threadpool::blocking;

pub fn execute_blocking_code<F, T, E>(f: F) -> impl Future<Item = T, Error = Error>
where
    F: FnOnce() -> Result<T, E>,
    E: Into<Error>,
{
    let mut f_opt = Some(f);
    poll_fn(move || blocking(|| (f_opt.take().unwrap())()))
        .map_err(Into::into)
        .and_then(|result| result.map_err(Into::into))
}
