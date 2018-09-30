use finchers::path;
use finchers::prelude::*;
use finchers_template::Renderer;
use std::sync::Arc;

fn main() {
    let engine = Arc::new(tera::compile_templates!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/**/*"
    )));

    let index = path!(@get /).and(Renderer::new(engine.clone(), "index.html"));

    let detail = path!(@get /"detail"/ String).wrap(Renderer::new(engine.clone(), "detail.html"));

    let p404 = endpoint::syntax::verb::get().and(Renderer::new(engine.clone(), "404.html"));

    let endpoint = index.or(detail).or(p404);

    finchers::launch(endpoint).start("127.0.0.1:4000")
}
