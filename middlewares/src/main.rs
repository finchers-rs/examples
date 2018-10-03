use finchers::output::body::Optional;
use finchers::prelude::*;
use finchers::rt::service::{Middleware, TowerWebMiddleware};

use futures::{Future, Poll};
use http::{Method, Request, Response};
use std::sync::Arc;

use tower_service::Service;
use tower_web::middleware::cors as tower_cors;
use tower_web::middleware::cors::{AllowedOrigins, CorsBuilder};
use tower_web::middleware::log::LogMiddleware;
use tower_web::util::http::{HttpMiddleware, HttpService};

fn main() {
    let endpoint = endpoint::cloned("Hello, world!");

    let log_middleware = Arc::new(TowerWebMiddleware::new(LogMiddleware::new(module_path!())));

    let cors_middleware = CorsMiddleware::new(
        CorsBuilder::new()
            .allow_origins(AllowedOrigins::Any { allow_null: true })
            .allow_methods(&[Method::GET])
            .build(),
    );

    println!("Listening on http://127.0.0.1:4000");
    finchers::rt::launch(endpoint)
        .with(log_middleware)
        .with(cors_middleware)
        .serve("127.0.0.1:4000")
        .expect("failed to start the server");
}

#[derive(Clone)]
struct CorsMiddleware(Arc<tower_cors::CorsMiddleware>);

impl CorsMiddleware {
    fn new(inner: tower_cors::CorsMiddleware) -> CorsMiddleware {
        CorsMiddleware(Arc::new(inner))
    }
}

impl<S> Middleware<S> for CorsMiddleware
where
    S: HttpService,
{
    type Request = Request<S::RequestBody>;
    type Response = Response<Optional<S::ResponseBody>>;
    type Error = S::Error;
    type Service = CorsService<S>;

    fn wrap(&self, service: S) -> Self::Service {
        CorsService(self.0.wrap_http(service))
    }
}

struct CorsService<S>(tower_cors::CorsService<S>);

impl<S> Service for CorsService<S>
where
    S: HttpService,
{
    type Request = Request<S::RequestBody>;
    type Response = Response<Optional<S::ResponseBody>>;
    type Error = S::Error;
    type Future = CorsFuture<S>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.0.poll_http_ready()
    }

    fn call(&mut self, request: Self::Request) -> Self::Future {
        CorsFuture(self.0.call_http(request))
    }
}

struct CorsFuture<S: HttpService>(<tower_cors::CorsService<S> as Service>::Future);

impl<S> Future for CorsFuture<S>
where
    S: HttpService,
{
    type Item = Response<Optional<S::ResponseBody>>;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0
            .poll()
            .map(|x| x.map(|response| response.map(Into::into)))
    }
}
