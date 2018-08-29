extern crate actix_web;
#[macro_use]
extern crate askama;
#[macro_use]
extern crate serde_derive;
extern crate serde;

use actix_web::{server, http::Method, App, Path, Result, HttpRequest, HttpResponse};
use askama::Template;

#[derive(Template, Serialize, Deserialize)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
    apples: u32,
}
fn hello(hello: Path<(HelloTemplate)>) -> Result<String> {
    Ok(hello.render().unwrap())
}

#[derive(Template, Serialize, Deserialize)]
#[template(path = "day.html")]
struct Day {
    id: u64,
}

#[derive(Template, Serialize, Deserialize)]
#[template(path = "index.html")]
struct Index {
    days: Vec<Day>,
}
fn index(_req: HttpRequest) -> HttpResponse {
    let mut index = Index {
        days: Vec::new(),
    };
    for i in 1..20 {
        index.days.push(Day { id:  i });
    }
    HttpResponse::Ok()
        .content_type("text/html")
        .body(index.render().unwrap())
}

fn main() {
    println!("I am running now!!!");
    server::new(|| App::new()
                .route("/index.html", Method::GET, index)
                .resource("/hello/{name}/{apples}/index.html",
                          |r| r.method(Method::GET).with(hello))
                )
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
