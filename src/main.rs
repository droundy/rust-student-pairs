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
    pairings: Vec<Pairing>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(path = "index.html")]
struct Index {
    days: Vec<Day>,
}

#[derive(Serialize, Deserialize)]
struct Student {
    id: u64,
}
#[derive(Serialize, Deserialize)]
struct Section {
    id: u64,
}
#[derive(Serialize, Deserialize)]
struct Team {
    id: u64,
}
#[derive(Serialize, Deserialize)]
enum Pairing {
    Pairing {
        section: Section,
        team: Team,
        primary: Student,
        secondary: Option<Student>,
    },
    Absent(Student),
}

#[derive(Serialize, Deserialize)]
struct Database {
    days: Vec<Day>,
    students: Vec<Student>,
    teams: Vec<Team>,
}

fn index(_req: HttpRequest) -> HttpResponse {
    let mut index = Index {
        days: Vec::new(),
    };
    for i in 1..20 {
        index.days.push(Day {
            id:  i,
            pairings: Vec::new(),
        });
    }
    HttpResponse::Ok()
        .content_type("text/html")
        .body(index.render().unwrap())
}

fn main() {
    println!("I am running now!!!");
    server::new(|| App::new()
                .route("/index.html", Method::GET, index)
                .route("/", Method::GET, index)
                .route("/day-{day}", Method::GET, index)
                .route("/grid-{day}", Method::GET, index)
                .route("/students", Method::GET, index)
                .route("/sections", Method::GET, index)
                .route("/teams", Method::GET, index)
                .resource("/hello/{name}/{apples}/index.html",
                          |r| r.method(Method::GET).with(hello))
                )
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
