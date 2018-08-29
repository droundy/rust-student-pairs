extern crate actix_web;
use actix_web::{server, http::Method, App, Path, Result};

#[macro_use]
extern crate askama;
use askama::Template;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
    apples: u32,
}

fn index(info: Path<(String, u32)>) -> Result<String> {
    let hello = HelloTemplate { name: info.0.clone(), apples: info.1 };
    Ok(hello.render().unwrap())
}

fn main() {
    server::new(|| App::new()
                .resource("/{username}/{id}/index.html",
                          |r| r.method(Method::GET).with(index))
                )
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
