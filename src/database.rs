use std::collections::HashSet;
// use std::hash::Hash;
use askama::Template;
use internment::Intern;
use atomicfile::AtomicFile;
use serde_yaml;

#[derive(Template,Serialize,Deserialize,Clone,Copy,PartialEq,Eq)]
#[template(path = "day.html")]
pub struct Day { id: usize }
impl Day {
    pub fn next(&self) -> Self {
        Day { id: self.id + 1 }
    }
    pub fn previous(&self) -> Self {
        if self.id == 0 {
            *self
        } else {
            Day { id: self.id - 1 }
        }
    }
}

#[derive(Template,Serialize,Deserialize,Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
#[template(path = "section.html")]
pub struct Section { pub name: Intern<String> }

#[derive(Template,Serialize,Deserialize,Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
#[template(path = "section.html")]
pub struct Student { pub name: Intern<String> }
impl From<String> for Student {
    fn from(s: String) -> Self {
        Student { name: Intern::new(s) }
    }
}

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
    pub fn has(&self, s: Student) -> bool {
        self.allocated_students().contains(&s)
    }
}

#[derive(Serialize,Deserialize,Clone,PartialEq,Eq)]
pub struct Data {
    pub students: HashSet<Student>,
    sections: HashSet<Section>,
    teams: HashSet<Team>,
    days: Vec<HashSet<Pairing>>,
}

impl Data {
    pub fn save(&self) {
        let f = AtomicFile::create("pairs.yaml")
            .expect("error creating save file");
        serde_yaml::to_writer(&f, self).expect("error writing yaml")
    }
    pub fn new() -> Self {
        if let Ok(f) = ::std::fs::File::open("pairs.yaml") {
            println!("Created file pairs.yaml...");
            if let Ok(s) = serde_yaml::from_reader::<_,Self>(&f) {
                return s;
            }
        }
        Data {
            days: Vec::new(),
            sections: HashSet::new(),
            students: HashSet::new(),
            teams: HashSet::new(),
        }
    }
    pub fn day(&mut self, day: Day) -> &HashSet<Pairing> {
        while day.id >= self.days.len() {
            self.days.push(HashSet::new());
        }
        return &self.days[day.id];
    }
    pub fn absent_students(&mut self, day: Day) -> Vec<Student> {
        if day.id >= self.days.len() {
            return Vec::new();
        }
        self.students.iter()
            .filter(|s| self.days[day.id].iter()
                    .any(|p| *p == Pairing::Absent((*s).clone())))
            .cloned()
            .collect()
    }
    pub fn unassigned_students(&self, day: &Day) -> Vec<Student> {
        if day.id >= self.days.len() {
            return Vec::new();
        }
        self.students.iter()
            .cloned()
            .filter(|&s| !self.days[day.id].iter().any(|p| p.has(s)))
            .collect()
    }

    pub fn new_student(&mut self, s: Student) {
        self.students.insert(s);
    }
    pub fn delete_student(&mut self, s: Student) {
        self.students.remove(&s);
    }
    pub fn rename_student(&mut self, old_s: Student, new_s: Student) {
        use database::Pairing::*;
        self.students.insert(new_s);
        self.students.remove(&old_s);
        for d in self.days.iter_mut() {
            let problems: Vec<_> = d.iter().cloned().filter(|&p| p.has(old_s)).collect();
            for mut p in problems {
                d.remove(&p);
                match p {
                    Absent(ref mut s) if *s == old_s => {
                        *s = new_s;
                    }
                    Pair { ref mut primary, .. } if *primary == old_s => {
                        *primary = new_s;
                    }
                    Pair { ref mut secondary, .. } if *secondary == old_s => {
                        *secondary = new_s;
                    }
                    Solo { ref mut student, .. } if *student == old_s => {
                        *student = new_s;
                    }
                    _ => (),
                }
                d.insert(p);
            }
        }
    }
}
