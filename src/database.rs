use std::collections::HashSet;
// use std::hash::Hash;
use askama::Template;
use internment::Intern;

#[derive(Template,Serialize,Deserialize,Clone,PartialEq,Eq)]
#[template(path = "day.html")]
pub struct Day {
    id: usize,
}

#[derive(Template,Serialize,Deserialize,Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
#[template(path = "section.html")]
pub struct Section { name: Intern<String> }

#[derive(Template,Serialize,Deserialize,Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
#[template(path = "section.html")]
pub struct Student { name: Intern<String> }

#[derive(Template,Serialize,Deserialize,Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
#[template(path = "section.html")]
pub struct Team { name: Intern<String> }

#[derive(Serialize,Deserialize,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub enum Pairing {
    Pair {
        section: Section,
        team: Team,
        primary: Student,
        secondary: Student,
    },
    Solo {
        section: Section,
        team: Team,
        student: Student,
    },
    Absent(Student),
}

impl Pairing {
    pub fn present_students(&self) -> Vec<Student> {
        use database::Pairing::*;
        match *self {
            Absent(_) => Vec::new(),
            Solo { student, .. } => vec![student],
            Pair { primary, secondary, .. } => vec![primary, secondary],
        }
    }
    pub fn allocated_students(&self) -> Vec<Student> {
        use database::Pairing::*;
        match *self {
            Absent( student ) => vec![student],
            Solo { student, .. } => vec![student],
            Pair { primary, secondary, .. } => vec![primary, secondary],
        }
    }
    pub fn has(&self, s: &Student) -> bool {
        self.present_students().contains(s)
    }
}

#[derive(Serialize,Deserialize,Clone,PartialEq,Eq)]
struct Data {
    students: HashSet<Student>,
    sections: HashSet<Section>,
    teams: HashSet<Team>,
    days: Vec<HashSet<Pairing>>,
}
