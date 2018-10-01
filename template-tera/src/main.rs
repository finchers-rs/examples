use finchers::path;
use finchers::prelude::*;
use finchers_template::renderer;
use std::sync::Arc;

const TEMPLATE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*");

fn main() {
    let engine = Arc::new(tera::compile_templates!(TEMPLATE_PATH));

    let index = path!(@get /).and(renderer(engine.clone(), "index.html"));

    let detail = path!(@get /"detail"/ String).wrap(renderer(engine.clone(), "detail.html"));

    let p404 = endpoint::syntax::verb::get().and(renderer(engine.clone(), "404.html"));

    let endpoint = index.or(detail).or(p404);

    finchers::launch(endpoint).start("127.0.0.1:4000")
}
