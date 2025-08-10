use crate::app::models::enrollment::Enrollment;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BulkEnrollmentImportRequest {
    pub enrollments: Vec<Enrollment>,
}

#[derive(Debug, Deserialize)]
pub struct EnrollmentCsvRow {
    pub student_id: i32,
    pub academic_year: String,
    pub grade_level: String,
    pub teacher_id: i32,
    #[serde(default)]
    pub status: String, //ignored - set to "Active" always for bulk update
    #[serde(default)]
    pub enrollment_date: String, //ignored - set in database as NOW()
    #[serde(default)]
    pub status_change_date: String, //ignored - set in database as NOW()
    #[serde(default)]
    pub notes: String,
}
