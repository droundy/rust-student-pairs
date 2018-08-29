#[macro_use]
extern crate rouille;
#[macro_use]
extern crate askama;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
extern crate tempfile;

mod atomicfile;

use rouille::{Request, Response};
use askama::Template;

#[derive(Template, Serialize, Deserialize)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
    apples: u32,
}
// fn hello(hello: Path<(HelloTemplate)>) -> Result<String> {
//     Ok(hello.render().unwrap())
// }

#[derive(Template, Serialize, Deserialize, Clone)]
#[template(path = "day.html")]
struct Day {
    id: usize,
    pairings: Vec<Pairing>,
}

#[derive(Template, Serialize, Deserialize, Clone, Debug)]
#[template(path = "student.html")]
struct Student {
    id: usize,
    name: String,
}
#[derive(Serialize, Deserialize, Clone)]
struct Section {
    id: usize,
}
#[derive(Serialize, Deserialize, Clone)]
struct Team {
    id: usize,
}
#[derive(Serialize, Deserialize, Clone)]
enum Pairing {
    Pairing {
        section: Section,
        team: Team,
        primary: Student,
        secondary: Option<Student>,
    },
    Absent(Student),
}

#[derive(Serialize, Deserialize, Clone)]
struct Database {
    days: Vec<Day>,
    sections: Vec<Section>,
    students: Vec<Student>,
    teams: Vec<Team>,
}
impl Database {
    fn save(&self) {
        let f = atomicfile::AtomicFile::create("pairs.yaml")
            .expect("error creating save file");
        serde_yaml::to_writer(&f, self).expect("error writing yaml")
    }
    fn new() -> Self {
        if let Ok(f) = ::std::fs::File::open("pairs.yaml") {
            println!("Created file pairs.yaml...");
            if let Ok(s) = serde_yaml::from_reader::<_,Self>(&f) {
                return s;
            }
        }
        Database {
            days: Vec::new(),
            sections: Vec::new(),
            students: Vec::new(),
            teams: Vec::new(),
        }
    }
}

#[derive(Template, Serialize, Deserialize)]
#[template(path = "index.html")]
struct Index {
    days: Vec<Day>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(path = "students.html")]
struct Students {
    students: Vec<Student>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NewStudent {
    name: String,
}
// fn new_student(form:  Form<NewStudent>) -> HttpResponse {
//     println!("{:?}", form);
//     let mut database = Database::new();
//     let student = Student {
//         id: database.students.len(),
//         name: form.name.clone(),
//     };
//     database.students.push(student);
//     database.save();
//     let page = Students {
//         students: database.students.clone(),
//     };
//     HttpResponse::Ok()
//         .content_type("text/html")
//         .body(page.render().unwrap())
// }

// fn change_student(form:  Form<Student>) -> HttpResponse {
//     println!("change {:?}", form);
//     let mut database = Database::new();
//     database.students[form.id].name = form.name.clone();
//     database.save();
//     let page = Students {
//         students: database.students.clone(),
//     };
//     HttpResponse::Ok()
//         .content_type("text/html")
//         .body(page.render().unwrap())
// }

fn main() {
    println!("I am running now!!!");
    rouille::start_server("0.0.0.0:8088", move |request| {
        let mut database = Database::new();
        let html = router!{
            request,
            (GET) (/) => {
                let page = Index {
                    days: database.days.clone(),
                };
                page.render().unwrap()
            },
            (GET) (/students) => {
                let page = Students {
                    students: database.students.clone(),
                };
                page.render().unwrap()
            },
            (POST) (/students) => {
                // let data = rouille::input::post::raw_urlencoded_post_input(request);
                // println!("data is {:?}", data);
                match post_input!(request, {
                    id: usize,
                    name: String,
                }) {
                    Ok(input) => {
                        if input.id < database.students.len() {
                            database.students[input.id] = Student {
                                id: input.id,
                                name: input.name,
                            };
                        } else {
                            let student = Student {
                                id: database.students.len(),
                                name: input.name,
                            };
                            database.students.push(student);
                        }
                    }
                    Err(e) => {
                        return Response::text(format!("Post students error: {:?}\n\n{:?}",
                                                      request, e));
                    }
                }
                let page = Students {
                    students: database.students.clone(),
                };
                page.render().unwrap()
            },
            _ => {
                format!("Error: {:?}", request)
            },
        };
        database.save();
        Response::html(html)
    });
    // server::new(|| App::new()
    //             // .middleware(
    //             //     csrf::CsrfFilter::build()
    //             //         .allowed_origin("https://www.example.com")
    //             //         .finish())
    //             .route("/index.html", Method::GET, index)
    //             .route("/", Method::GET, index)
    //             .route("/day-{day}", Method::GET, index)
    //             .route("/grid-{day}", Method::GET, index)
    //             .route("/students", Method::GET, students)
    //             .route("/sections", Method::GET, index)
    //             .route("/teams", Method::GET, index)

    //             .route("/students", Method::POST, change_student)
    //             .route("/students", Method::POST, post_student_error)
    //             .route("/students", Method::POST, new_student)
    //             .resource("/hello/{name}/{apples}/index.html",
    //                       |r| r.method(Method::GET).with(hello))
    //             )
    //     .bind("127.0.0.1:8088")
    //     .unwrap()
    //     .run();
}
