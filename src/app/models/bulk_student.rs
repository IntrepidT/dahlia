use crate::app::models::student::{ESLEnum, GenderEnum, GradeEnum, InterventionEnum, Student};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BulkStudentImportRequest {
    pub students: Vec<Student>,
}

#[derive(Debug, Deserialize)]
pub struct StudentCsvRow {
    pub firstname: String,
    pub lastname: String,
    pub preferred: String,
    //note that these are strings now but get converted to GenderEnum later
    pub gender: String,
    //same as above
    pub date_of_birth: String,
    pub student_id: i32,
    //same as above
    pub esl: String,
    //same as above
    pub current_grade_level: String,
    pub teacher: String,
    pub iep: bool,
    pub bip: bool,
    pub student_504: bool,
    pub readplan: bool,
    pub gt: bool,
    pub intervention: String,
    pub eye_glasses: bool,
    pub notes: String,
    pub pin: i32,
}

#[derive(Debug, Deserialize)]
pub struct StudentCsvRowAlternative {
    pub firstname: String,
    pub lastname: String,
    pub preferred: String,
    pub gender: String,
    pub date_of_birth: String,
    pub student_id: i32,
    pub esl: String,
    pub grade: String,
    pub teacher: String,
    pub iep: bool,
    pub bip: bool,
    pub student_504: bool,
    pub readplan: bool,
    pub gt: bool,
    pub intervention: String,
    pub eye_glasses: bool,
    pub notes: String,
    pub pin: i32,
}

impl From<StudentCsvRowAlternative> for StudentCsvRow {
    fn from(row: StudentCsvRowAlternative) -> Self {
        StudentCsvRow {
            firstname: row.firstname,
            lastname: row.lastname,
            preferred: row.preferred,
            gender: row.gender,
            date_of_birth: row.date_of_birth,
            student_id: row.student_id,
            esl: row.esl,
            current_grade_level: row.grade,
            teacher: row.teacher,
            iep: row.iep,
            bip: row.bip,
            student_504: row.student_504,
            readplan: row.readplan,
            gt: row.gt,
            intervention: row.intervention,
            eye_glasses: row.eye_glasses,
            notes: row.notes,
            pin: row.pin,
        }
    }
}
