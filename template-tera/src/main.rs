use finchers::path;
use finchers::prelude::*;
use finchers_template_tera::template;

fn main() {
    let template = template(tera::compile_templates!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/**/*"
    )));

    let index = path!(@get /).and(template.to_renderer("index.html"));

    let detail = path!(@get /"detail"/ String).wrap(template.to_renderer("detail.html"));

    let p404 = endpoint::syntax::verb::get().and(template.to_renderer("404.html"));

    let endpoint = index.or(detail).or(p404);

    finchers::launch(endpoint).start("127.0.0.1:4000")
}
