use finchers::endpoint::wrapper::Wrapper;
use finchers::endpoint::{Context, Endpoint, EndpointResult};
use finchers::error::Error;

use failure::SyncFailure;
use futures::{Future, Poll};
use http::Response;
use serde::Serialize;
use std::sync::Arc;
use tera::Tera;

pub fn template(engine: Arc<Tera>, name: impl Into<String>) -> Template {
    Template {
        engine,
        name: name.into(),
    }
}

#[derive(Debug, Clone)]
pub struct Template {
    engine: Arc<Tera>,
    name: String,
}

impl<'a, E, CtxT> Wrapper<'a, E> for Template
where
    E: Endpoint<'a, Output = (CtxT,)>,
    CtxT: Serialize,
{
    type Output = (Response<String>,);
    type Endpoint = TemplateEndpoint<E>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        TemplateEndpoint {
            endpoint,
            engine: self.engine,
            name: self.name,
        }
    }
}

#[derive(Debug)]
pub struct TemplateEndpoint<E> {
    endpoint: E,
    engine: Arc<Tera>,
    name: String,
}

impl<'a, E, CtxT> Endpoint<'a> for TemplateEndpoint<E>
where
    E: Endpoint<'a, Output = (CtxT,)>,
    CtxT: Serialize,
{
    type Output = (Response<String>,);
    type Future = TemplateFuture<'a, E>;

    #[inline]
    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(TemplateFuture {
            future: self.endpoint.apply(cx)?,
            endpoint: self,
        })
    }
}

#[derive(Debug)]
pub struct TemplateFuture<'a, E: Endpoint<'a>> {
    future: E::Future,
    endpoint: &'a TemplateEndpoint<E>,
}

impl<'a, E, CtxT> Future for TemplateFuture<'a, E>
where
    E: Endpoint<'a, Output = (CtxT,)>,
    CtxT: Serialize,
{
    type Item = (Response<String>,);
    type Error = Error;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let ctx = futures::try_ready!(self.future.poll());
        self.endpoint
            .engine
            .render(&self.endpoint.name, &ctx)
            .map(|body| {
                let response = Response::builder()
                    .header("content-type", "text/html; charset=utf-8")
                    .body(body)
                    .expect("should be a valid response");
                (response,).into()
            }).map_err(|err| finchers::error::fail(SyncFailure::new(err)))
    }
}
