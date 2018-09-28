use finchers::prelude::*;
use finchers::{path, routes};
use std::sync::Arc;

mod template;
use crate::template::template;

fn main() {
    let engine = Arc::new(tera::compile_templates!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/**/*"
    )));

    let index = path!(@get /)
        .map(|| ())
        .wrap(template(engine.clone(), "index.html"));

    let detail = path!(@get /"detail"/)
        .map(|| ())
        .wrap(template(engine.clone(), "detail.html"));

    let p404 = endpoint::syntax::verb::get()
        .map(|| ())
        .wrap(template(engine.clone(), "404.html"));

    let endpoint = routes![index, detail, p404];

    finchers::launch(endpoint).start("127.0.0.1:4000")
}
