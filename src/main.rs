extern crate actix_web;
#[macro_use]
extern crate askama;
#[macro_use]
extern crate serde_derive;
extern crate serde;

use actix_web::{server, http::Method, App, Path, Result};
use askama::Template;

#[derive(Template, Serialize, Deserialize)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
    apples: u32,
}

fn index(hello: Path<(HelloTemplate)>) -> Result<String> {
    Ok(hello.render().unwrap())
}

fn main() {
    server::new(|| App::new()
                .resource("/{name}/{apples}/index.html",
                          |r| r.method(Method::GET).with(index))
                )
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
