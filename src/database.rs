use std::collections::HashSet;
// use std::hash::Hash;
use askama::Template;
use internment::Intern;
use atomicfile::AtomicFile;
use serde_yaml;
use std::str::FromStr;

#[derive(Template,Serialize,Deserialize,Clone,Copy,PartialEq,Eq)]
#[template(path = "day.html")]
pub struct Day { pub id: usize }
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
impl FromStr for Day {
    type Err = <usize as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        usize::from_str(s).map(|x| Day { id: x })
    }
}

#[derive(Template,Serialize,Deserialize,Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
#[template(path = "section.html")]
pub struct Section { pub name: Intern<String> }
impl From<String> for Section {
    fn from(s: String) -> Self {
        Section { name: Intern::new(s) }
    }
}

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
pub struct Team { pub name: Intern<String> }
impl From<String> for Team {
    fn from(s: String) -> Self {
        Team { name: Intern::new(s) }
    }
}

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
    pub fn full_pair(&self) -> bool {
        if let Pairing::Pair { .. } = *self {
            true
        } else {
            false
        }
    }
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
    pub fn section(&self) -> Option<Section> {
        use database::Pairing::*;
        match *self {
            Absent( _ ) => None,
            Solo { section, .. } => Some(section),
            Pair { section, .. } => Some(section),
        }
    }
    pub fn team(&self) -> Option<Team> {
        use database::Pairing::*;
        match *self {
            Absent( _ ) => None,
            Solo { team, .. } => Some(team),
            Pair { team, .. } => Some(team),
        }
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
    pub fn student_options(&self, day: Day) -> Vec<StudentOptions> {
        let pairings = if day.id >= self.days.len() {
            HashSet::new()
        } else {
            self.days[day.id].clone()
        };
        let mut options = Vec::new();
        for s in self.list_students().iter().cloned() {
            let current_pairing = pairings.iter().filter(|p| p.has(s)).cloned().next();
            let mut opt = StudentOptions {
                day: day,
                student: s,
                current_pairing: current_pairing,
                possible_teams: Vec::new(),
            };
            for t in self.teams.iter() {
                if pairings.iter().filter(|p| !(p.team() == Some(*t) && p.full_pair()))
                    .next().is_none()
                {
                    opt.possible_teams.push(*t);
                }
            }
            opt.possible_teams.sort();
            options.push(opt);
        }
        options
    }
    pub fn absent_students(&self, day: Day) -> Vec<Student> {
        if day.id >= self.days.len() {
            return Vec::new();
        }
        self.students.iter()
            .filter(|s| self.days[day.id].iter()
                    .any(|p| *p == Pairing::Absent((*s).clone())))
            .cloned()
            .collect()
    }
    pub fn unassigned_students(&self, day: Day) -> Vec<Student> {
        if day.id >= self.days.len() {
            return Vec::new();
        }
        self.students.iter()
            .cloned()
            .filter(|&s| !self.days[day.id].iter().any(|p| p.has(s)))
            .collect()
    }

    pub fn list_days(&self) -> Vec<Day> {
        (0..self.days.len()).map(|i| Day { id: i }).collect()
    }
    pub fn add_day(&mut self) {
        self.days.push(HashSet::new());
    }
    pub fn list_students(&self) -> Vec<Student> {
        let mut list: Vec<_> = self.students.iter().cloned().collect();
        list.sort();
        list
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
                    Absent(ref mut s) => {
                        *s = new_s;
                    }
                    Solo { ref mut student, .. } => {
                        *student = new_s;
                    }
                    Pair { ref mut primary, .. } if *primary == old_s => {
                        *primary = new_s;
                    }
                    Pair { ref mut secondary, .. } if *secondary == old_s => {
                        *secondary = new_s;
                    }
                    _ => (),
                }
                d.insert(p);
            }
        }
    }

    pub fn list_sections(&self) -> Vec<Section> {
        let mut list: Vec<_> = self.sections.iter().cloned().collect();
        list.sort();
        list
    }
    pub fn new_section(&mut self, s: Section) {
        self.sections.insert(s);
    }
    pub fn delete_section(&mut self, s: Section) {
        self.sections.remove(&s);
        for d in self.days.iter_mut() {
            d.retain(|p| p.section() != Some(s));
        }
    }
    pub fn rename_section(&mut self, old_s: Section, new_s: Section) {
        use database::Pairing::*;
        self.sections.insert(new_s);
        self.sections.remove(&old_s);
        for d in self.days.iter_mut() {
            let problems: Vec<_> = d.iter().cloned().filter(|&p| p.section() == Some(old_s)).collect();
            for mut p in problems {
                d.remove(&p);
                match p {
                    Pair { ref mut section, .. } => {
                        *section = new_s;
                    }
                    Solo { ref mut section, .. } => {
                        *section = new_s;
                    }
                    _ => (),
                }
                d.insert(p);
            }
        }
    }

    pub fn list_teams(&self) -> Vec<Team> {
        let mut list: Vec<_> = self.teams.iter().cloned().collect();
        list.sort();
        list
    }
    pub fn new_team(&mut self, s: Team) {
        self.teams.insert(s);
    }
    pub fn delete_team(&mut self, s: Team) {
        self.teams.remove(&s);
        for d in self.days.iter_mut() {
            d.retain(|p| p.team() != Some(s));
        }
    }
    pub fn rename_team(&mut self, old_s: Team, new_s: Team) {
        use database::Pairing::*;
        self.teams.insert(new_s);
        self.teams.remove(&old_s);
        for d in self.days.iter_mut() {
            let problems: Vec<_> = d.iter().cloned().filter(|&p| p.team() == Some(old_s)).collect();
            for mut p in problems {
                d.remove(&p);
                match p {
                    Pair { ref mut team, .. } => {
                        *team = new_s;
                    }
                    Solo { ref mut team, .. } => {
                        *team = new_s;
                    }
                    _ => (),
                }
                d.insert(p);
            }
        }
    }
}


#[derive(Template, Serialize, Deserialize, Clone)]
#[template(path = "student-options.html")]
pub struct StudentOptions {
    pub day: Day,
    pub student: Student,
    pub current_pairing: Option<Pairing>,
    pub possible_teams: Vec<Team>,
}

impl StudentOptions {
    fn current_team(&self) -> Team {
        match self.current_pairing {
            None => Team { name: Intern::new("unassigned".to_string()) },
            Some(Pairing::Absent(_)) => Team { name: Intern::new("absent".to_string()) },
            Some(Pairing::Solo { team, .. }) => team,
            Some(Pairing::Pair { team, .. }) => team,
        }
    }
}
