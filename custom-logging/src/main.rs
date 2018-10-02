use finchers::output::status::Created;
use finchers::prelude::*;
use finchers::{path, routes};

use finchers::endpoints::logging::{logging_fn, Info};

use http::Response;
use slog::Logger;
use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::types::{Format, Severity};
use sloggers::Build;

fn build_logger() -> Logger {
    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    builder.destination(Destination::Stdout);
    builder.format(Format::Full);
    builder.build().expect("failed to construct a Logger")
}

fn main() {
    let endpoint = routes![
        path!(@get / "index").map(|| "Index page"),
        path!(@get / "created").map(|| Created("created")),
    ];

    // modify the above endpoint to convert all errors into an HTTP response.
    //
    // FIXME: provide it as a single Wrapper.
    let endpoint = endpoint
        .wrap(endpoint::wrapper::or_reject())
        .wrap(endpoint::wrapper::recover(|err| {
            let mut response = Response::builder()
                .status(err.status_code())
                .body(err.to_string())
                .unwrap();
            err.headers(response.headers_mut());
            futures::future::ok(response)
        }));

    let logger = build_logger();

    // A Wrapper for logging.
    let logging = logging_fn({
        let logger = logger.clone();
        move |info: Info| {
            slog::info!(logger, "response";
                "request_method" => info.input.method().to_string(),
                "request_uri" => info.input.uri().to_string(),
                "response_status" => info.status.to_string(),
                "response_time" => format!("{:?}", info.start.elapsed()),
            );
        }
    });

    // Wrap the endpoint with the specified logging wrapper.
    let endpoint = endpoint.wrap(logging);

    slog::info!(logger, "starting server";
        "local_addr" => "http://127.0.0.1:4000"
    );
    finchers::launch(endpoint).start("127.0.0.1:4000");
}
