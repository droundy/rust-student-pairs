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

use database::{Student, Day, Team, Section, StudentOptions};

#[derive(Template, Serialize, Deserialize, Clone)]
#[template(path = "edit-day.html")]
struct EditDay {
    today: Day,
    unassigned: Vec<Student>,
    absent: Vec<Student>,
    all: Vec<StudentOptions>,
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
        let mut data = database::Data::new();
        let html = router!{
            request,
            (GET) (/) => {
                let page = Index {
                    days: data.list_days(),
                };
                page.render().unwrap()
            },
            (POST) (/) => {
                data.add_day();
                let page = Index {
                    days: data.list_days(),
                };
                page.render().unwrap()
            },
            (GET) (/day/{today: Day}) => {
                let page = EditDay {
                    today: today,
                    unassigned: data.unassigned_students(today),
                    absent: data.absent_students(today),
                    all: data.student_options(today),
                };
                page.render().unwrap()
            },
            (GET) (/students) => {
                let page = Students {
                    students: data.list_students(),
                };
                page.render().unwrap()
            },
            (POST) (/students) => {
                match post_input!(request, {
                    oldname: String,
                    newname: String,
                }) {
                    Ok(input) => {
                        if input.oldname == "" {
                            data.new_student(Student::from(input.newname));
                        } else if input.newname == "" {
                            data.delete_student(Student::from(input.oldname));
                        } else {
                            data.rename_student(Student::from(input.oldname),
                                                Student::from(input.newname));
                        }
                    }
                    Err(e) => {
                        return Response::text(format!("Post students error: {:?}\n\n{:?}",
                                                      request, e));
                    }
                }
                let page = Students {
                    students: data.list_students(),
                };
                page.render().unwrap()
            },
            (GET) (/sections) => {
                let page = Sections {
                    sections: data.list_sections(),
                };
                page.render().unwrap()
            },
            (POST) (/sections) => {
                match post_input!(request, {
                    oldname: String,
                    newname: String,
                }) {
                    Ok(input) => {
                        if input.oldname == "" {
                            data.new_section(Section::from(input.newname));
                        } else if input.newname == "" {
                            data.delete_section(Section::from(input.oldname));
                        } else {
                            data.rename_section(Section::from(input.oldname),
                                                Section::from(input.newname));
                        }
                    }
                    Err(e) => {
                        return Response::text(format!("Post sections error: {:?}\n\n{:?}",
                                                      request, e));
                    }
                }
                let page = Sections {
                    sections: data.list_sections(),
                };
                page.render().unwrap()
            },
            (GET) (/teams) => {
                let page = Teams {
                    teams: data.list_teams(),
                };
                page.render().unwrap()
            },
            (POST) (/teams) => {
                match post_input!(request, {
                    oldname: String,
                    newname: String,
                }) {
                    Ok(input) => {
                        if input.oldname == "" {
                            data.new_team(Team::from(input.newname));
                        } else if input.newname == "" {
                            data.delete_team(Team::from(input.oldname));
                        } else {
                            data.rename_team(Team::from(input.oldname),
                                                Team::from(input.newname));
                        }
                    }
                    Err(e) => {
                        return Response::text(format!("Post teams error: {:?}\n\n{:?}",
                                                      request, e));
                    }
                }
                let page = Teams {
                    teams: data.list_teams(),
                };
                page.render().unwrap()
            },
            _ => {
                format!("Error: {:?}", request)
            },
        };
        data.save();
        Response::html(html)
    });
}
