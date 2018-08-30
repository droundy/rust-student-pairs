#[macro_use]
extern crate rouille;
#[macro_use]
extern crate askama;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
extern crate tempfile;
extern crate internment;

mod atomicfile;
pub mod database;

use rouille::{Response};
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

#[derive(Template, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[template(path = "day.html")]
struct Day {
    id: usize,
    pairings: Vec<Pairing>,
}
#[derive(Template, Serialize, Deserialize, Clone)]
#[template(path = "edit-day.html")]
struct EditDay {
    today: Day,
    unassigned: Vec<Student>,
    absent: Vec<Student>,
}

#[derive(Template, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[template(path = "student.html")]
struct Student {
    id: usize,
    name: String,
}
#[derive(Template, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[template(path = "student.html")]
struct Section {
    id: usize,
    name: String,
}
#[derive(Template, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[template(path = "student.html")]
struct Team {
    id: usize,
    name: String,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
enum Pairing {
    Pairing {
        section: Section,
        team: Team,
        primary: Student,
        secondary: Option<Student>,
    },
    Absent(Student),
}
impl Pairing {
    fn has(&self, s: &Student) -> bool {
        match *self {
            Pairing::Absent(ref x) => s == x,
            Pairing::Pairing {primary: ref x, secondary: None, ..} => {
                x == s
            }
            Pairing::Pairing {primary: ref x, secondary: Some(ref y), ..} => {
                x == s || y == s
            }
        }
    }
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
    fn absent_students(&self, day: &Day) -> Vec<Student> {
        self.students.iter()
            .filter(|s| day.pairings.iter().any(|p| *p == Pairing::Absent((*s).clone())))
            .cloned()
            .collect()
    }
    fn unassigned_students(&self, day: &Day) -> Vec<Student> {
        self.students.iter()
            .filter(|s| !day.pairings.iter().any(|p| p.has(s)))
            .cloned()
            .collect()
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

#[derive(Template, Serialize, Deserialize)]
#[template(path = "sections.html")]
struct Sections {
    sections: Vec<Section>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(path = "teams.html")]
struct Teams {
    teams: Vec<Team>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NewStudent {
    name: String,
}
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
            (POST) (/) => {
                let day = Day {
                    id: database.days.len(),
                    pairings: Vec::new(),
                };
                database.days.push(day);
                let page = Index {
                    days: database.days.clone(),
                };
                page.render().unwrap()
            },
            (GET) (/day/{daynum: usize}) => {
                if daynum >= database.days.len() {
                    let page = Index {
                        days: database.days.clone(),
                    };
                    page.render().unwrap()
                } else {
                    let page = EditDay {
                        today: database.days[daynum].clone(),
                        unassigned: database.unassigned_students(&database.days[daynum]),
                        absent: database.absent_students(&database.days[daynum]),
                    };
                    page.render().unwrap()
                }
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
            (GET) (/sections) => {
                let page = Sections {
                    sections: database.sections.clone(),
                };
                page.render().unwrap()
            },
            (POST) (/sections) => {
                // let data = rouille::input::post::raw_urlencoded_post_input(request);
                // println!("data is {:?}", data);
                match post_input!(request, {
                    id: usize,
                    name: String,
                }) {
                    Ok(input) => {
                        if input.id < database.sections.len() {
                            database.sections[input.id] = Section {
                                id: input.id,
                                name: input.name,
                            };
                        } else {
                            let section = Section {
                                id: database.sections.len(),
                                name: input.name,
                            };
                            database.sections.push(section);
                        }
                    }
                    Err(e) => {
                        return Response::text(format!("Post sections error: {:?}\n\n{:?}",
                                                      request, e));
                    }
                }
                let page = Sections {
                    sections: database.sections.clone(),
                };
                page.render().unwrap()
            },
            (GET) (/teams) => {
                let page = Teams {
                    teams: database.teams.clone(),
                };
                page.render().unwrap()
            },
            (POST) (/teams) => {
                // let data = rouille::input::post::raw_urlencoded_post_input(request);
                // println!("data is {:?}", data);
                match post_input!(request, {
                    id: usize,
                    name: String,
                }) {
                    Ok(input) => {
                        if input.id < database.teams.len() {
                            database.teams[input.id] = Team {
                                id: input.id,
                                name: input.name,
                            };
                        } else {
                            let team = Team {
                                id: database.teams.len(),
                                name: input.name,
                            };
                            database.teams.push(team);
                        }
                    }
                    Err(e) => {
                        return Response::text(format!("Post teams error: {:?}\n\n{:?}",
                                                      request, e));
                    }
                }
                let page = Teams {
                    teams: database.teams.clone(),
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
}
