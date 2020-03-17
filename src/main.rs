#[macro_use]
extern crate rouille;
extern crate askama;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
extern crate tempfile;
extern crate internment;
extern crate rand;

mod atomicfile;
pub mod database;

use rouille::{Response};
use askama::Template;

use database::{Student, Day, Team, Section, Zoom, StudentOptions, TeamOptions};

#[derive(Template, Serialize, Deserialize, Clone)]
#[template(path = "edit-day.html")]
struct EditDay {
    today: Day,
    unassigned: Vec<Student>,
    absent: Vec<Student>,
    all: Vec<StudentOptions>,
}

#[derive(Template, Serialize, Deserialize, Clone)]
#[template(path = "team-view.html")]
struct TeamView {
    today: Day,
    unassigned: Vec<Student>,
    absent: Vec<Student>,
    all: Vec<(Section, Vec<TeamOptions>)>,
}

#[derive(Template, Serialize, Deserialize, Clone)]
#[template(path = "section-view.html")]
struct SectionView {
    today: Day,
    unassigned: Vec<Student>,
    absent: Vec<Student>,
    all: Vec<(Section, Vec<TeamOptions>, Zoom)>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(path = "index.html")]
struct Index {
    days: Vec<Day>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(path = "students.html")]
struct Students {
    sections: Vec<(Section, Vec<Student>)>,
    focus_section: Section,
}

#[derive(Template, Serialize, Deserialize)]
#[template(path = "sections.html")]
struct Sections {
    sections: Vec<(Section, Zoom)>,
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
        let response = rouille::match_assets(&request, "static");
        if response.is_success() {
            return response;
        }
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
                match post_input!(request, {
                    id: usize,
                    name: String,
                }) {
                    Ok(input) => {
                        if input.id == data.list_days().len() {
                            data.add_day();
                        }
                        if input.name != "" {
                            data.name_day(input.id, input.name);
                        } else {
                            // By process of elimination, the check
                            // button must have been hit.
                            data.toggle_lock_day(Day::from(input.id));
                        }
                    }
                    Err(e) => {
                        return Response::text(format!("Post / error: {:?}\n\n{:?}",
                                                      request, e));
                    }
                }
                let page = Index {
                    days: data.list_days(),
                };
                page.render().unwrap()
            },
            (GET) (/day/{today: Day}) => {
                let today = data.improve_day(today);
                let all: Vec<_> = data.student_options(today).into_iter()
                    .flat_map(|(_,v)| v).collect();
                let page = EditDay {
                    today: today,
                    unassigned: data.unassigned_students(today),
                    absent: data.absent_students(today),
                    all,
                };
                page.render().unwrap()
            },
            (POST) (/day/{today: Day}) => {
                let today = data.improve_day(today);
                if !data.day_unlocked(today) {
                    return Response::text(format!("Cannot modify locked day: {}",
                                                  today.pretty()));
                }
                match post_input!(request, {
                    team: String,
                    section: String,
                    student: String,
                    action: String,
                }) {
                    Ok(input) => {
                        let section = Section::from(input.section);
                        if input.action == "student" {
                            println!("assigning {} to {:?} {:?}", input.student,
                                     section, input.team);
                            data.assign_student(today,
                                                Student::from(input.student),
                                                section,
                                                Team::from(input.team));
                        } else if input.action == "Shuffle" {
                            println!("Shuffling {}...", section);
                            data.shuffle(today, section);
                        } else if input.action == "Shuffle with continuity" {
                            println!("Shuffling with continuity {}...", section);
                            data.shuffle_with_continuity(today, section);
                        } else if input.action == "Repeat" {
                            println!("Repeating {}...", section);
                            data.repeat(today, section);
                        } else if input.action == "Clear all" {
                            println!("I should be clearing all...");
                            for s in data.students_present_in_section(today, section) {
                                data.unpair_student(today, s);
                            }
                        } else if input.action == "Grand shuffle" {
                            println!("I should be doing grand shuffle...");
                            data.grand_shuffle(today);
                        } else if input.action == "Grand shuffle with continuity" {
                            println!("I should be doing grand shuffle with continuity...");
                            data.grand_shuffle_with_continuity(today);
                        } else {
                            println!("What do I do with action {:?}?", input.action);
                        }
                    }
                    Err(e) => {
                        return Response::text(format!("Post students error: {:?}\n\n{:?}",
                                                      request, e));
                    }
                }
                let all: Vec<_> = data.student_options(today).into_iter()
                    .flat_map(|(_,v)| v).collect();
                let page = EditDay {
                    today: today,
                    unassigned: data.unassigned_students(today),
                    absent: data.absent_students(today),
                    all,
                };
                page.render().unwrap()
            },
            (GET) (/pairs/{today: Day}) => {
                let today = data.improve_day(today);
                let page = TeamView {
                    today: today,
                    unassigned: data.unassigned_students(today),
                    absent: data.absent_students(today),
                    all: data.team_options(today),
                };
                page.render().unwrap()
            },
            (GET) (/sections/{today: Day}) => {
                let today = data.improve_day(today);
                let mut unassigned = data.unassigned_students(today);
                let absent = data.absent_students(today);
                unassigned.retain(|s| !absent.contains(s));
                let zooms = data.get_zooms();
                let page = SectionView {
                    today: today,
                    unassigned,
                    absent,
                    all: data.team_options(today).into_iter()
                        .map(|(sec,stu)| (sec, stu, zooms[&sec]))
                        .collect(),
                };
                page.render().unwrap()
            },
            (POST) (/pairs/{today: Day}) => {
                let today = data.improve_day(today);
                if !data.day_unlocked(today) {
                    return Response::text(format!("Cannot modify locked day: {}",
                                                  today.pretty()));
                }
                match post_input!(request, {
                    team: String,
                    section: String,
                    primary: String,
                    secondary: String,
                    action: String,
                }) {
                    Ok(input) => {
                        let section = Section::from(input.section);
                        if input.action == "team" {
                            let team = Team::from(input.team);
                            data.unpair_team(today, team);
                            if input.primary != "" {
                                data.assign_student(today,
                                                    Student::from(input.primary),
                                                    section, team);
                            }
                            if input.secondary != "" {
                                data.assign_student(today,
                                                    Student::from(input.secondary),
                                                    section, team);
                            }
                        } else if input.action == "Shuffle" {
                            println!("Shuffling {}...", section);
                            data.shuffle(today, section);
                        } else if input.action == "Shuffle with continuity" {
                            println!("Shuffling with continuity {}...", section);
                            data.shuffle_with_continuity(today, section);
                        } else if input.action == "Repeat" {
                            println!("Repeating {}...", section);
                            data.repeat(today, section);
                        } else if input.action == "Clear all" {
                            println!("I should be clearing all...");
                            for s in data.students_present_in_section(today, section) {
                                data.unpair_student(today, s);
                            }
                        } else if input.action == "Grand shuffle" {
                            println!("I should be doing grand shuffle...");
                            data.grand_shuffle(today);
                        } else if input.action == "Grand shuffle with continuity" {
                            println!("I should be doing grand shuffle with continuity...");
                            data.grand_shuffle_with_continuity(today);
                        } else {
                            println!("What do I do with foolish action {:?}?", input.action);
                        }
                    }
                    Err(e) => {
                        return Response::text(format!("Post students error: {:?}\n\n{:?}",
                                                      request, e));
                    }
                }
                let page = TeamView {
                    today: today,
                    unassigned: data.unassigned_students(today),
                    absent: data.absent_students(today),
                    all: data.team_options(today),
                };
                page.render().unwrap()
            },
            (GET) (/students) => {
                let page = Students {
                    sections: data.list_students_by_section(),
                    focus_section: Section::from("".to_string()),
                };
                page.render().unwrap()
            },
            (POST) (/students) => {
                let focus_section;
                match post_input!(request, {
                    section: String,
                    oldname: String,
                    newname: String,
                }) {
                    Ok(input) => {
                        focus_section = Section::from(input.section.clone());
                        if input.oldname == "" {
                            data.new_student(Student::from(input.newname),
                                             Section::from(input.section));
                        } else if input.newname == "" {
                            data.delete_student(Student::from(input.oldname));
                        } else {
                            data.rename_student(Student::from(input.oldname),
                                                Student::from(input.newname),
                                                Section::from(input.section));
                        }
                    }
                    Err(e) => {
                        return Response::text(format!("Post students error: {:?}\n\n{:?}",
                                                      request, e));
                    }
                }
                let page = Students {
                    sections: data.list_students_by_section(),
                    focus_section: focus_section,
                };
                page.render().unwrap()
            },
            (GET) (/sections) => {
                let page = Sections {
                    sections: data.zoom_sections(),
                };
                page.render().unwrap()
            },
            (POST) (/sections) => {
                match post_input!(request, {
                    oldname: String,
                    newname: String,
                    newzoom: String,
                }) {
                    Ok(input) => {
                        println!("posted to sections...");
                        let zoom = Zoom::from(input.newzoom);
                        if input.oldname == "" {
                            data.new_section(Section::from(input.newname), zoom);
                        } else if input.newname == "" {
                            data.delete_section(Section::from(input.oldname));
                        } else {
                            println!("renaming... {} {} {:?}",
                                     input.oldname, input.newname, zoom);
                            data.rename_section(Section::from(input.oldname),
                                                Section::from(input.newname),
                                                zoom);
                        }
                    }
                    Err(e) => {
                        return Response::text(format!("Post sections error: {:?}\n\n{:?}",
                                                      request, e));
                    }
                }
                let page = Sections {
                    sections: data.zoom_sections(),
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
