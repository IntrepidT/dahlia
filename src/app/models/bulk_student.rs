use crate::app::models::student::{ESLEnum, GenderEnum, GradeEnum, Student};
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
    pub grade: String,
    pub teacher: String,
    pub iep: bool,
    pub bip: bool,
    pub student_504: bool,
    pub readplan: bool,
    pub gt: bool,
    pub intervention: bool,
    pub eye_glasses: bool,
    pub notes: String,
}
