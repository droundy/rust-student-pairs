use std::collections::{HashSet, HashMap};
// use std::hash::Hash;
use askama::Template;
use internment::Intern;
use atomicfile::AtomicFile;
use serde_yaml;
use std::str::FromStr;
use rand::{thread_rng};
use rand::seq::SliceRandom;

#[derive(Template,Serialize,Deserialize,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
#[template(path = "day.html")]
pub struct Day {
    pub id: usize,
    #[serde(default)]
    pub name: Option<Intern<String>>,
    #[serde(default)]
    pub unlocked: bool,
}
impl Day {
    pub fn next(&self) -> Self {
        Day::from(self.id + 1)
    }
    pub fn previous(&self) -> Self {
        if self.id == 0 {
            *self
        } else {
            Day::from(self.id - 1)
        }
    }
    pub fn pretty(&self) -> String {
        if let Some(n) = self.name {
            (*n).clone()
        } else {
            format!("Day {}", self.id)
        }
    }
}
impl From<usize> for Day {
    fn from(x: usize) -> Day {
        Day { id: x, name: None, unlocked: false }
    }
}
impl FromStr for Day {
    type Err = <usize as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        usize::from_str(s).map(|x| Day::from(x))
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
#[template(path = "zoom.html")]
pub struct Zoom { pub id: Intern<String> }
impl From<String> for Zoom {
    fn from(s: String) -> Self {
        Zoom { id: Intern::new(s) }
    }
}
impl Zoom {
    pub fn url(&self) -> String {
        let clean = self.id.replace(&[' ', '-'][..], "");
        clean
        // format!("https://oregonstate.zoom.us/j/{}", clean)
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

#[derive(Serialize,Deserialize,Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
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
    Unassigned {
        section: Section,
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
    pub fn assigned_students(&self) -> Vec<Student> {
        use database::Pairing::*;
        match *self {
            Absent(_) => Vec::new(),
            Solo { student, .. } => vec![student],
            Unassigned { .. } => Vec::new(),
            Pair { primary, secondary, .. } => vec![secondary, primary],
        }
    }
    pub fn present_students(&self) -> Vec<Student> {
        use database::Pairing::*;
        match *self {
            Absent(_) => Vec::new(),
            Solo { student, .. } => vec![student],
            Unassigned { student, .. } => vec![student],
            Pair { primary, secondary, .. } => vec![secondary, primary],
        }
    }
    pub fn allocated_students(&self) -> Vec<Student> {
        use database::Pairing::*;
        match *self {
            Absent( student ) => vec![student],
            Solo { student, .. } => vec![student],
            Unassigned { student, .. } => vec![student],
            Pair { primary, secondary, .. } => vec![primary, secondary],
        }
    }
    fn has(&self, s: Student) -> bool {
        self.allocated_students().contains(&s)
    }
    pub fn section(&self) -> Option<Section> {
        use database::Pairing::*;
        match *self {
            Absent( _ ) => None,
            Solo { section, .. } => Some(section),
            Unassigned { section, .. } => Some(section),
            Pair { section, .. } => Some(section),
        }
    }
    pub fn team(&self) -> Option<Team> {
        use database::Pairing::*;
        match *self {
            Absent( _ ) => None,
            Unassigned { .. } => None,
            Solo { team, .. } => Some(team),
            Pair { team, .. } => Some(team),
        }
    }
}

#[derive(Serialize,Deserialize,Clone,PartialEq,Eq)]
pub struct Data {
    #[serde(default)]
    course_path: String,
    #[serde(default)]
    student_sections: HashMap<Student, Section>,
    #[serde(default)]
    sections: HashMap<Section, Zoom>,
    #[serde(default)]
    teams: HashSet<Team>,
    #[serde(default)]
    days: Vec<HashSet<Pairing>>,
    #[serde(default)]
    daynames: HashMap<usize, Intern<String>>,
    #[serde(default)]
    days_unlocked: HashSet<usize>,
}

impl Data {
    pub fn save(&self) {
        let f = AtomicFile::create(format!("{}.yaml", self.course_path))
            .expect("error creating save file");
        serde_yaml::to_writer(&f, self).expect("error writing yaml")
    }
    pub fn new(path: &str) -> Self {
        assert!(path.chars().all(char::is_alphanumeric));
        assert!(path.len() > 10);
        if let Ok(f) = ::std::fs::File::open(format!("{}.yaml", path)) {
            match serde_yaml::from_reader::<_,Self>(&f) {
                Ok(s) => {
                    return s;
                }
                Err(e) => {
                    println!("Error reading: {:?}", e);
                }
            }
        }
        Data {
            course_path: path.to_string(),
            days: Vec::new(),
            sections: HashMap::new(),
            student_sections: HashMap::new(),
            teams: HashSet::new(),
            daynames: HashMap::new(),
            days_unlocked: HashSet::new(),
        }
    }
    pub fn day(&mut self, day: Day) -> &HashSet<Pairing> {
        while day.id >= self.days.len() {
            self.days.push(HashSet::new());
        }
        return &self.days[day.id];
    }
    pub fn improve_day(&self, day: Day) -> Day {
        if let Some(&n) = self.daynames.get(&day.id) {
            return Day { id: day.id, name: Some(n), unlocked: self.day_unlocked(day) };
        }
        day
    }
    pub fn name_day(&mut self, id: usize, name: String) {
        self.daynames.insert(id, Intern::new(name));
    }
    fn nonrepeat_partners_for_day(&self, day: Day, s1: Student, s2: Student) -> bool {
        for d in 0..day.id {
            if self.days[d].iter().any(|p| p.present_students().contains(&s1)
                                           && p.present_students().contains(&s2)) {
                return false;
            }
        }
        true
    }
    fn pick_partner_from(&self, day: Day, s1: Student, options: &mut Vec<Student>)
                         -> Option<Student> {
        if let Some(s2) = options.iter().cloned()
            .filter(|&s2| self.nonrepeat_partners_for_day(day, s1, s2))
            .next()
        {
            *options = options.iter().cloned().filter(|&s| s != s2).collect();
            Some(s2)
        } else {
            options.pop()
        }
    }
    pub fn students_present_in_section(&self, day: Day, _section: Section)
                                       -> Vec<Student> {
        // let mut students: Vec<Student> = self.student_sections.iter()
        //     .filter(|(_,&sec)| sec == section)
        //     .map(|(&s,_)| s)
        //     .collect();
        // for p in self.days[day.id].iter().cloned() {
        //     if p.section() == Some(section) {
        //         students.extend(p.present_students());
        //     } else {
        //         // If they are in a *different* section *or* are
        //         // absent, then they are not going to be present in
        //         // *this* section.
        //         for s in p.allocated_students().into_iter() {
        //             remove_student_from_vec(s, &mut students);
        //         }
        //     }
        // }
        // students.sort();
        // students.dedup();

        // FIXME: This function lies, since with remote teaching and
        // all sections simultaneous I can move students between
        // section willy-nilly.
        let mut students = self.list_students();
        let absent = self.absent_students(day);
        students.retain(|s| !absent.contains(s));
        students
    }
    pub fn shuffle_sections(&mut self, day: Day) {
        let mut sections: Vec<_> = self.sections.keys().cloned().collect();
        sections.sort();
        let mut pairings: Vec<_> = self.days[day.id].drain().collect();
        println!("we have {} total pairings", pairings.len());
        self.days[day.id].extend(pairings.iter().filter(|p| !p.team().is_some()).cloned());
        pairings.retain(|p| p.team().is_some());
        println!("we have {} pairings that have a team", pairings.len());
        pairings.sort_by_key(|p| p.team());
        for (p, section) in split_evenly(&pairings, sections.len()).zip(sections.into_iter()) {
            for pairing in p.iter().cloned() {
                match pairing {
                    Pairing::Pair { team, primary, secondary, .. } => {
                        self.days[day.id].insert(Pairing::Pair { team, primary, secondary, section });
                    }
                    Pairing::Solo { team, student, ..  } => {
                        self.days[day.id].insert(Pairing::Solo { team, student, section });
                    }
                    x => {
                        self.days[day.id].insert(x);
                    }
                }
            }
        }
    }
    pub fn grand_shuffle(&mut self, day: Day) {
        let section = self.sections.keys().cloned().next().expect("Oops, need a section");
        let absent: Vec<_> = self.absent_students(day);
        let students: Vec<_> = self.student_sections.keys().cloned()
            .filter(|s| !absent.contains(s)).collect();
        for student in students.into_iter() {
            self.unassign_student(day, student);
            self.days[day.id].insert(Pairing::Unassigned { student, section });
        }

        self.shuffle(day, section);
        self.shuffle_sections(day);
    }
    pub fn grand_shuffle_with_continuity(&mut self, day: Day) {
        let section = self.sections.keys().cloned().next().expect("Oops, need a section");
        let absent: Vec<_> = self.absent_students(day);
        let mut students: Vec<_> = self.student_sections.keys().cloned()
            .filter(|s| !absent.contains(s)).collect();
        for &student in students.iter() {
            self.unassign_student(day, student);
            self.days[day.id].insert(Pairing::Unassigned { student, section });
        }
        students.shuffle(&mut thread_rng());
        let mut last_week_pairs: Vec<_> =
            if day.id > 0 {
                self.days[day.id-1].iter().cloned()
                    .filter(|p| p.team().is_some())
                    .collect()
            } else {
                Vec::new()
            };
        last_week_pairs.shuffle(&mut thread_rng());
        self.days[day.id] = absent.into_iter().map(|s| Pairing::Absent(s)).collect();
        let mut possible_teams: Vec<_> = self.teams.iter().cloned().collect();
        let mut newpairings = Vec::new();
        for p in last_week_pairs.into_iter() {
            let team = p.team().unwrap();
            for student in p.present_students() {
                if remove_student_from_vec(student, &mut students).is_some() {
                    possible_teams.retain(|&t| t != team);
                    newpairings.push(Pairing::Solo {team, student, section});
                    break;
                }
            }
        }
        for mut p in newpairings.into_iter() {
            let primary = p.present_students()[0];
            let team = p.team().unwrap();
            if let Some(secondary) = self.pick_partner_from(day, primary, &mut students) {
                p = Pairing::Pair { primary, secondary, section, team };
            }
            self.days[day.id].insert(p);
        }
        possible_teams.sort();
        possible_teams.reverse();
        while students.len() > 1 && possible_teams.len() > 0 {
            let primary = students.pop().unwrap();
            let team = possible_teams.pop().unwrap();
            let secondary = self.pick_partner_from(day, primary, &mut students).unwrap();
            self.days[day.id].insert(Pairing::Pair { primary, secondary, section, team });
        }
        if let Some(student) = students.pop() {
            if let Some(team) = possible_teams.pop() {
                self.days[day.id].insert(Pairing::Solo { student, team, section });
            }
        }

        self.shuffle_sections(day);
    }
    pub fn shuffle(&mut self, day: Day, section: Section) {
        let mut students: Vec<Student> = self.students_present_in_section(day, section);
        students.shuffle(&mut thread_rng());
        self.days[day.id] = self.days[day.id].iter().cloned()
            .filter(|p| p.section() != Some(section)).collect();
        let mut possible_teams: Vec<_> =
            self.teams.iter().cloned()
            .filter(|&t| !self.days[day.id].iter().any(|p| p.team() == Some(t))).collect();
        possible_teams.sort();
        possible_teams.reverse();
        while students.len() > 1 && possible_teams.len() > 0 {
            let primary = students.pop().unwrap();
            let team = possible_teams.pop().unwrap();
            let secondary = self.pick_partner_from(day, primary, &mut students).unwrap();
            self.days[day.id].insert(Pairing::Pair { primary, secondary, section, team });
        }
        if let Some(student) = students.pop() {
            if let Some(team) = possible_teams.pop() {
                self.days[day.id].insert(Pairing::Solo { student, team, section });
            }
        }
    }
    pub fn shuffle_with_continuity(&mut self, day: Day, section: Section) {
        let mut students: Vec<Student> = self.students_present_in_section(day, section);
        students.shuffle(&mut thread_rng());
        let last_week_pairs: Vec<_> =
            if day.id > 0 {
                self.days[day.id-1].iter().cloned()
                    .filter(|p| p.section() == Some(section) && p.team().is_some())
                    .collect()
            } else {
                Vec::new()
            };
        self.days[day.id].retain(|p| p.section() != Some(section));
        let mut possible_teams: Vec<_> =
            self.teams.iter().cloned()
            .filter(|&t| !self.days[day.id].iter().any(|p| p.team() == Some(t))).collect();
        let mut last_week_pairs: Vec<_> =
            last_week_pairs.iter().cloned()
            .filter(|p| possible_teams.contains(&p.team().unwrap()))
            .collect();
        last_week_pairs.shuffle(&mut thread_rng());
        let mut newpairings = Vec::new();
        for p in last_week_pairs.into_iter() {
            let team = p.team().unwrap();
            for st in p.present_students() {
                if let Some(student) = remove_student_from_vec(st, &mut students) {
                    possible_teams = possible_teams.iter().cloned()
                        .filter(|&t| t != team).collect();
                    newpairings.push(Pairing::Solo {team, student, section});
                    break;
                }
            }
        }
        for mut p in newpairings.into_iter() {
            let primary = p.present_students()[0];
            let team = p.team().unwrap();
            if let Some(secondary) = self.pick_partner_from(day, primary, &mut students) {
                p = Pairing::Pair { primary, secondary, section, team };
            }
            self.days[day.id].insert(p);
        }
        possible_teams.sort();
        possible_teams.reverse();
        while students.len() > 1 && possible_teams.len() > 0 {
            let primary = students.pop().unwrap();
            let team = possible_teams.pop().unwrap();
            let secondary = self.pick_partner_from(day, primary, &mut students).unwrap();
            self.days[day.id].insert(Pairing::Pair { primary, secondary, section, team });
        }
        if let Some(student) = students.pop() {
            if let Some(team) = possible_teams.pop() {
                self.days[day.id].insert(Pairing::Solo { student, team, section });
            }
        }
    }
    pub fn repeat(&mut self, day: Day, section: Section) {
        let mut students: Vec<Student> = self.students_present_in_section(day, section);
        students.shuffle(&mut thread_rng());
        let last_week_pairs: Vec<_> =
            if day.id > 0 {
                self.days[day.id-1].iter().cloned()
                    .filter(|p| p.section() == Some(section) && p.team().is_some())
                    .collect()
            } else {
                Vec::new()
            };
        self.days[day.id] = self.days[day.id].iter().cloned()
            .filter(|p| p.section() != Some(section)).collect();
        let mut possible_teams: Vec<_> =
            self.teams.iter().cloned()
            .filter(|&t| !self.days[day.id].iter().any(|p| p.team() == Some(t))).collect();
        let mut last_week_pairs: Vec<_> =
            last_week_pairs.iter().cloned()
            .filter(|p| possible_teams.contains(&p.team().unwrap()))
            .collect();
        last_week_pairs.shuffle(&mut thread_rng());
        let mut newpairings = Vec::new();
        for p in last_week_pairs.iter().cloned() {
            println!("   {:?}", p);
        }
        for p in last_week_pairs.into_iter() {
            let team = p.team().unwrap();
            match p {
                Pairing::Pair { primary, secondary, .. } => {
                    if remove_student_from_vec(primary, &mut students).is_some() {
                        if remove_student_from_vec(secondary, &mut students).is_some() {
                            println!("{} and {} are still in {}", primary, secondary, team);
                            newpairings.push(Pairing::Pair {
                                primary, secondary, section, team });
                            possible_teams = possible_teams.iter().cloned()
                                .filter(|&t| t != team).collect();
                        } else {
                            println!("{} is alas alone in {}", primary, team);
                            newpairings.push(Pairing::Solo { student: primary, section, team });
                            possible_teams = possible_teams.iter().cloned()
                                .filter(|&t| t != team).collect();
                        }
                    } else {
                        if remove_student_from_vec(secondary, &mut students).is_some() {
                            println!(">>> {} is now alone in {} ({} dropped)", secondary, team, primary);
                            newpairings.push(Pairing::Solo { student: secondary,
                                                             section, team });
                            possible_teams = possible_teams.iter().cloned()
                                .filter(|&t| t != team).collect();
                        }
                    }
                }
                Pairing::Solo { student, .. } => {
                    if let Some(student) = remove_student_from_vec(student, &mut students) {
                        println!("{} is still alone in {}", student, team);
                        newpairings.push(Pairing::Solo { student, section, team });
                        possible_teams = possible_teams.iter().cloned()
                            .filter(|&t| t != team).collect();
                    }
                }
                _ => (),
            }
        }
        for mut p in newpairings.into_iter() {
            if let Pairing::Solo { student, team, .. } = p {
                if let Some(secondary) = self.pick_partner_from(day, student, &mut students) {
                    println!("choosing {} to go with {} in {}",
                             secondary, student, team);
                    p = Pairing::Pair { primary: student, secondary, section, team };
                }
            }
            self.days[day.id].insert(p);
        }
        possible_teams.sort();
        possible_teams.reverse();
        println!("still have remaining {} students", students.len());
        while students.len() > 1 && possible_teams.len() > 0 {
            let primary = students.pop().unwrap();
            let team = possible_teams.pop().unwrap();
            let secondary = self.pick_partner_from(day, primary, &mut students).unwrap();
            self.days[day.id].insert(Pairing::Pair { primary, secondary, section, team });
        }
        if let Some(student) = students.pop() {
            if let Some(team) = possible_teams.pop() {
                self.days[day.id].insert(Pairing::Solo { student, team, section });
            }
        }
    }
    pub fn team_options(&self, day: Day) -> Vec<(Section, Vec<TeamOptions>)> {
        let mut section_options = Vec::new();
        for section in self.sections.keys().cloned() {
            let mut teams = Vec::new();
            let present_students = self.students_present_in_section(day, section);
            let unassigned: Vec<_> = self.unassigned_students(day).iter().cloned()
                .filter(|s| present_students.contains(s)).collect();
            for p in self.days[day.id].iter().cloned()
                .filter(|p| p.section() == Some(section))
            {
                let previous_students = if day.id > 0 {
                    self.days[day.id-1].iter()
                        .filter(|pp| pp.team() == p.team())
                        .map(|p| p.present_students())
                        .next().unwrap_or(Vec::new())
                } else {
                    Vec::new()
                };

                match p {
                    Pairing::Pair { team, primary, secondary, .. } => {
                        let mut primary_options = unassigned.clone();
                        primary_options.push(primary);
                        primary_options.sort();
                        let primary_options = primary_options.into_iter()
                            .map(|s| (s,Vec::new()))
                            .map(|(s, mut tags)| {
                                if !self.nonrepeat_partners_for_day(day, secondary, s) {
                                    tags.push("repeat".to_string());
                                }
                                if previous_students.contains(&s) {
                                    tags.push("reuser".to_string());
                                }
                                (s,tags)
                            })
                            .collect();
                        let mut secondary_options = unassigned.clone();
                        secondary_options.push(secondary);
                        secondary_options.sort();
                        let secondary_options = secondary_options.into_iter()
                            .map(|s| (s,Vec::new()))
                            .map(|(s, mut tags)| {
                                if !self.nonrepeat_partners_for_day(day, primary, s) {
                                    tags.push("repeat".to_string());
                                }
                                if previous_students.contains(&s) {
                                    tags.push("reuser".to_string());
                                }
                                (s,tags)
                            })
                            .collect();
                        teams.push(TeamOptions {
                            day, team, section,
                            primary: Choices {
                                current: Some(primary),
                                possibilities: primary_options,
                                choice_name: "primary".to_string(),
                                tags: Vec::new(),
                            }.normalize(),
                            secondary: Choices {
                                current: Some(secondary),
                                possibilities: secondary_options,
                                choice_name: "secondary".to_string(),
                                tags: Vec::new(),
                            }.normalize(),
                            current_pairing: p,
                        });
                    }
                    Pairing::Solo { team, student, .. } => {
                        let mut primary_options = unassigned.clone();
                        primary_options.push(student);
                        primary_options.sort();
                        let primary_options = primary_options.into_iter()
                            .map(|s| (s,Vec::new()))
                            .map(|(s, mut tags)| {
                                if previous_students.contains(&s) {
                                    tags.push("reuser".to_string());
                                }
                                (s,tags)
                            })
                            .collect();
                        let secondary_options = unassigned.iter().cloned()
                            .map(|s| (s,Vec::new()))
                            .map(|(s, mut tags)| {
                                if !self.nonrepeat_partners_for_day(day, student, s) {
                                    tags.push("repeat".to_string());
                                }
                                if previous_students.contains(&s) {
                                    tags.push("reuser".to_string());
                                }
                                (s,tags)
                            })
                            .collect();
                        teams.push(TeamOptions {
                            day, team, section,
                            primary: Choices {
                                current: Some(student),
                                possibilities: primary_options,
                                choice_name: "primary".to_string(),
                                tags: Vec::new(),
                            }.normalize(),
                            secondary: Choices {
                                current: None,
                                possibilities: secondary_options,
                                choice_name: "secondary".to_string(),
                                tags: Vec::new(),
                            }.normalize(),
                            current_pairing: p,
                        });
                    }
                    _ => (),
                }
            }
            teams.sort();
            section_options.push((section, teams));
        }
        section_options.sort();
        section_options
    }
    pub fn student_options(&self, day: Day) -> Vec<(Section, Vec<StudentOptions>)> {
        let pairings = if day.id >= self.days.len() {
            HashSet::new()
        } else {
            self.days[day.id].clone()
        };
        let mut section_options = Vec::new();
        for (section, students) in self.list_students_by_section().iter().cloned() {
            let mut options = Vec::new();
            for s in students.iter().cloned() {
                let current_pairing = pairings.iter().filter(|p| p.has(s)).cloned().next();
                let previous_team = if day.id > 0 {
                    self.days[day.id-1].iter()
                        .filter(|p| p.has(s) && p.full_pair())
                        .map(|p| p.team().unwrap())
                        .next()
                } else {
                    None
                };
                let mut opt = StudentOptions {
                    day: day,
                    student: s,
                    current_pairing: current_pairing,
                    possible_teams: Vec::new(),
                    possible_sections: self.sections.keys().cloned().collect(),
                    default_section: self.student_sections[&s],
                    previous_team: previous_team,
                };
                for t in self.teams.iter() {
                    if pairings.iter()
                        .filter(|p| p.team() == Some(*t))
                        .filter(|p| !p.has(s))
                        .filter(|p| p.full_pair() || (p.section().is_some()
                                                      && opt.current_section().is_some()
                                                      && p.section() != opt.current_section()))
                        .next().is_none()
                    {
                        // This team is not full, and it doesn't exist in
                        // a section other than the current one, so it is
                        // one this student can join.
                        opt.possible_teams.push(*t);
                    }
                }
                opt.possible_teams.sort();
                opt.possible_sections.sort();
                options.push(opt);
            }
            options.sort_by_key(|o| o.current_section());
            section_options.push((section, options));
        }
        section_options
    }
    pub fn absent_students(&self, day: Day) -> Vec<Student> {
        if day.id >= self.days.len() {
            return Vec::new();
        }
        self.student_sections.keys()
            .filter(|s| self.days[day.id].iter()
                    .any(|p| *p == Pairing::Absent((*s).clone())))
            .cloned()
            .collect()
    }
    pub fn unassigned_students(&self, day: Day) -> Vec<Student> {
        if day.id >= self.days.len() {
            return Vec::new();
        }
        self.student_sections.keys()
            .cloned()
            .filter(|&s| !self.days[day.id].iter()
                    .any(|p| p.assigned_students().contains(&s)))
            .collect()
    }

    pub fn list_days(&self) -> Vec<Day> {
        (0..self.days.len()).map(|i| self.improve_day(Day::from(i))).collect()
    }
    pub fn add_day(&mut self) {
        self.days.push(HashSet::new());
    }
    pub fn list_students(&self) -> Vec<Student> {
        let mut list: Vec<_> = self.student_sections.keys().cloned().collect();
        list.sort();
        list
    }
    pub fn list_students_by_section(&self) -> Vec<(Section, Vec<Student>)> {
        let mut list = Vec::new();
        for section in self.sections.keys().cloned() {
            let mut students: Vec<Student> = self.student_sections.iter()
                .filter(|(_,&sec)| sec == section)
                .map(|(&s,_)| s)
                .collect();
            students.sort();
            list.push((section, students));
        }
        list.sort();
        list
    }
    pub fn toggle_lock_day(&mut self, day: Day) {
        if self.day_unlocked(day) {
            self.days_unlocked.remove(&day.id);
        } else {
            self.days_unlocked.insert(day.id);
        }
    }
    pub fn day_unlocked(&self, day: Day) -> bool {
        self.days_unlocked.contains(&day.id)
    }
    fn unassign_student(&mut self, day: Day, student: Student) {
        let mut newpairings: HashSet<_> =
            self.days[day.id].iter().cloned()
            .filter(|p| !p.has(student))
            .collect();
        match self.days[day.id].iter().cloned().filter(|p| p.has(student)).next() {
            Some(Pairing::Pair{ primary, secondary, team, section }) => {
                if primary != student {
                    newpairings.insert(Pairing::Solo {
                        student: primary,
                        team,
                        section
                    });
                } else {
                    newpairings.insert(Pairing::Solo {
                        student: secondary,
                        team,
                        section
                    });
                }
            }
            _ => (),
        }
        self.days[day.id] = newpairings;
    }
    pub fn assign_student(&mut self, day: Day, student: Student,
                          section: Section, team: Team) {
        if section == Section::from("".to_string()) {
            println!("Marking {} as absent", student);
            self.unassign_student(day, student);
            self.days[day.id].insert(Pairing::Absent(student));
        } else if team == Team::from("".to_string()) {
            println!("Marking {} as unassigned in section {}", student, section);
            self.unassign_student(day, student);
            self.days[day.id].insert(Pairing::Unassigned { student, section });
            println!("Things: {:?}", self.days[day.id]);
        } else {
            println!("Should mark {} as on team {} in section {}", student, team, section);
            self.unassign_student(day, student);
            match self.days[day.id].iter().cloned().filter(|p| p.team() == Some(team)).next() {
                Some(Pairing::Pair{ .. }) => {
                    println!("Team is already full.  :(");
                }
                Some(Pairing::Solo{ student: primary, team, section: oldsec }) => {
                    if oldsec != section {
                        println!("Not possible: sections do not match.");
                    } else {
                        self.unassign_student(day, primary);
                        self.days[day.id].insert(Pairing::Pair {
                            primary: primary,
                            secondary: student,
                            team: team,
                            section: section,
                        });
                        println!("Adding {} to team with {}", student, primary);
                    }
                }
                _ => {
                    self.days[day.id].insert(Pairing::Solo { student, team, section });
                }
            }
        }
    }
    pub fn unpair_student(&mut self, day: Day, student: Student) {
        let section = match self.days[day.id].iter().cloned().filter(|p| p.has(student)).next() {
            Some(Pairing::Pair { section, .. }) => section,
            Some(Pairing::Solo { section, .. }) => section,
            Some(Pairing::Unassigned { section, .. }) => section,
            Some(Pairing::Absent(_)) => { return; }
            None => { return; }
        };
        self.unassign_student(day, student);
        self.days[day.id].insert(Pairing::Unassigned { student, section });
    }
    pub fn unpair_team(&mut self, day: Day, team: Team) {
        match self.days[day.id].iter().cloned().filter(|p| p.team() == Some(team)).next() {
            Some(Pairing::Pair { primary, secondary, .. }) => {
                self.unpair_student(day, primary);
                self.unpair_student(day, secondary);
            }
            Some(Pairing::Solo { student, .. }) => {
                self.unpair_student(day, student);
            }
            _ => (),
        };
    }
    pub fn new_student(&mut self, s: Student, section: Section) {
        self.student_sections.insert(s, section);
    }
    pub fn delete_student(&mut self, s: Student) {
        self.student_sections.remove(&s);
    }
    pub fn rename_student(&mut self, old_s: Student, new_s: Student, section: Section) {
        use database::Pairing::*;
        self.student_sections.insert(new_s, section);
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
                    Unassigned { ref mut student, .. } => {
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
        let mut list: Vec<_> = self.sections.keys().cloned().collect();
        list.sort();
        list
    }
    pub fn zoom_sections(&self) -> Vec<(Section, Zoom)> {
        let mut list: Vec<_> = self.sections.iter().map(|(a,b)| (a.clone(), b.clone())).collect();
        list.sort();
        list
    }
    pub fn new_section(&mut self, s: Section, zoom: Zoom) {
        self.sections.insert(s, zoom);
    }
    pub fn delete_section(&mut self, s: Section) {
        self.sections.remove(&s);
        for d in self.days.iter_mut() {
            d.retain(|p| p.section() != Some(s));
        }
    }
    pub fn rename_section(&mut self, old_s: Section, new_s: Section, zoom: Zoom) {
        use database::Pairing::*;
        self.sections.remove(&old_s);
        self.sections.insert(new_s, zoom);
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
                    Unassigned { ref mut section, .. } => {
                        *section = new_s;
                    }
                    _ => (),
                }
                d.insert(p);
            }
        }
    }

    pub fn get_zooms(&self) -> HashMap<Section, Zoom> {
        self.sections.clone()
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
    pub default_section: Section,
    pub previous_team: Option<Team>,
    pub possible_sections: Vec<Section>,
}

impl StudentOptions {
    fn is_current_team(&self, t: &Team) -> bool {
        match self.current_pairing {
            None => false,
            Some(Pairing::Absent(_)) => false,
            Some(Pairing::Unassigned { .. }) => false,
            Some(Pairing::Solo { team, .. }) => team == *t,
            Some(Pairing::Pair { team, .. }) => team == *t,
        }
    }
    fn is_current_section(&self, s: &Section) -> bool {
        Some(*s) == self.current_section()
    }
    fn is_previous_team(&self, t: &Team) -> bool {
        Some(*t) == self.previous_team
    }
    fn is_repeating_team(&self) -> bool {
        self.previous_team.map(|t| self.is_current_team(&t)).unwrap_or(false)
    }
    fn current_section(&self) -> Option<Section> {
        match self.current_pairing {
            None => Some(self.default_section),
            Some(Pairing::Absent(_)) => None,
            Some(Pairing::Unassigned { section, .. }) => Some(section),
            Some(Pairing::Solo { section, .. }) => Some(section),
            Some(Pairing::Pair { section, .. }) => Some(section),
        }
    }
    fn tags(&self) -> Vec<String> {
        let mut tags = Vec::new();
        if self.is_repeating_team() {
            tags.push("reuser".to_string());
        }
        tags
    }
}

fn remove_student_from_vec(s: Student, options: &mut Vec<Student>) -> Option<Student> {
    if !options.contains(&s) {
        return None;
    }
    *options = options.iter().cloned().filter(|&s2| s2 != s).collect();
    Some(s)
}


#[derive(Template, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[template(path = "choices.html")]
pub struct Choices<T: ::std::fmt::Display + Eq + Clone> {
    pub current: Option<T>,
    /// A list of possible choices as well as their tags.
    pub possibilities: Vec<(T, Vec<String>)>,
    pub choice_name: String,
    /// Tags to apply to the entire select (i.e. regarding the current).
    pub tags: Vec<String>,
}

impl<T: Eq + Clone + ::std::fmt::Display> Choices<T> {
    pub fn normalize(mut self) -> Self {
        if let Some(c) = self.current.clone() {
            if let Some((_, tags)) = self.possibilities.iter().cloned()
                .filter(|(o,_)| o.clone() == c.clone()).next()
            {
                self.tags.extend(tags);
            }
        }
        self
    }
    pub fn is_current(&self, x: T) -> bool {
        if let Some(ref c) = self.current {
            c == &x
        } else {
            false
        }
    }
    pub fn current_string(&self) -> String {
        self.current.clone().map(|x| format!("{}", x)).unwrap_or("-".to_string())
    }
}

#[derive(Template, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[template(path = "team-options.html")]
pub struct TeamOptions {
    pub day: Day,
    pub team: Team,
    pub section: Section,
    pub primary: Choices<Student>,
    pub secondary: Choices<Student>,
    pub current_pairing: Pairing,
}

// impl TeamOptions {
//     fn is_on_team(&self, s: &Student) -> bool {
//         self.primary.is_current(*s) || self.secondary.is_current(*s)
//     }
// }

fn split_evenly<T>(slice: &[T], n: usize) -> impl Iterator<Item = &[T]> {
    struct Iter<'a, I> {
        pub slice: &'a [I],
        pub n: usize,
        pub am_odd: bool,
    }
    impl<'a, I> Iterator for Iter<'a, I> {
        type Item = &'a [I];
        fn next(&mut self) -> Option<&'a [I]> {
            if self.slice.len() == 0 {
                return None;
            }
            let leftovers = self.slice.len() % self.n;
            let extra = if leftovers == 0 {
                0
            } else if rand::random::<usize>() % self.n < leftovers {
                1
            } else {
                0
            };
            let (first, rest) = self.slice.split_at(self.slice.len() / self.n + extra);
            self.slice = rest;
            self.n -= 1;
            Some(first)
        }
    }
    Iter { slice, n, am_odd: slice.len() & 1 == 1 }
}

#[test]
fn test_split_evenly() {
    let eight: [usize; 8] = [1,2,3,4,5,6,7,8];
    let chunks: Vec<_> = split_evenly(&eight, 3).collect();
    assert_eq!(&eight[..], &chunks.iter().flat_map(|x| *x).cloned().collect::<Vec<usize>>()[..]);
    assert_eq!(chunks.iter().map(|c| c.len()).max(), Some(3));
    assert_eq!(chunks.iter().map(|c| c.len()).min(), Some(2));

    let chunks: Vec<_> = split_evenly(&eight, 5).collect();
    assert_eq!(&eight[..], &chunks.iter().flat_map(|x| *x).cloned().collect::<Vec<usize>>()[..]);
    assert_eq!(chunks.iter().map(|c| c.len()).max(), Some(2));
    assert_eq!(chunks.iter().map(|c| c.len()).min(), Some(1));

    let seven = [1,2,3,4,5,6,7];
    let chunks: Vec<_> = split_evenly(&seven, 3).collect();
    assert_eq!(&seven[..], &chunks.iter().flat_map(|x| *x).cloned().collect::<Vec<usize>>()[..]);
    assert_eq!(chunks.iter().map(|c| c.len()).max(), Some(3));
    assert_eq!(chunks.iter().map(|c| c.len()).min(), Some(2));

    let chunks: Vec<_> = split_evenly(&seven, 5).collect();
    assert_eq!(&seven[..], &chunks.iter().flat_map(|x| *x).cloned().collect::<Vec<usize>>()[..]);
    assert_eq!(chunks.iter().map(|c| c.len()).max(), Some(2));
    assert_eq!(chunks.iter().map(|c| c.len()).min(), Some(1));


    let nineteen: [usize; 19] = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19];

    let chunks: Vec<_> = split_evenly(&nineteen, 5).collect();
    assert_eq!(&nineteen[..], &chunks.iter().flat_map(|x| *x).cloned().collect::<Vec<usize>>()[..]);
    assert_eq!(chunks.iter().map(|c| c.len()).max(), Some(4));
    assert_eq!(chunks.iter().map(|c| c.len()).min(), Some(3));

    let mut last_was_small = false;
    for _ in 0..60 {
        let chunks: Vec<_> = split_evenly(&nineteen, 5).collect();
        assert_eq!(&nineteen[..], &chunks.iter().flat_map(|x| *x).cloned().collect::<Vec<usize>>()[..]);
        assert_eq!(chunks.iter().map(|c| c.len()).max(), Some(4));
        assert_eq!(chunks.iter().map(|c| c.len()).min(), Some(3));

        println!("  {:?}", chunks.iter().map(|c| c.len()).collect::<Vec<_>>());
        if chunks[4].len() == 3 {
            last_was_small = true;
        }
    }
    assert!(last_was_small);
}
