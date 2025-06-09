use crate::app::models::bulk_student::StudentCsvRow;
use crate::app::models::student::{
    AddStudentRequest, ESLEnum, GenderEnum, GradeEnum, InterventionEnum,
};
use chrono::NaiveDate;
use csv::ReaderBuilder;
use leptos::*;
use std::str::FromStr;
use validator::Validate;

#[cfg(feature = "ssr")]
use {crate::app::db::student_database, sqlx::PgPool};

#[server(UploadStudentsBulk, "/api")]
pub async fn upload_students_bulk(file_contents: String) -> Result<usize, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Parse and validate all students first
        let students = parse_and_validate_students(&file_contents)?;

        // Bulk insert using optimized method
        match student_database::bulk_insert_students_optimized(students, &pool).await {
            Ok(count) => Ok(count),
            Err(e) => {
                log::error!("Bulk student import failed: {:?}", e);
                Err(ServerFnError::ServerError(format!("Import failed: {}", e)))
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError(
            "Server-side functionality not available".to_string(),
        ))
    }
}

// Separate parsing logic for reusability and better error handling
fn parse_and_validate_students(
    file_contents: &str,
) -> Result<Vec<AddStudentRequest>, ServerFnError> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file_contents.as_bytes());

    let mut students = Vec::new();
    let mut errors = Vec::new();

    for (row_num, result) in rdr.deserialize::<StudentCsvRow>().enumerate() {
        match result {
            Ok(record) => {
                match parse_student_record(record, row_num + 2) {
                    // +2 for header and 1-based indexing
                    Ok(student) => students.push(student),
                    Err(e) => errors.push(e),
                }
            }
            Err(e) => {
                errors.push(format!("Row {}: CSV parsing error: {}", row_num + 2, e));
            }
        }
    }

    if !errors.is_empty() {
        let error_msg = format!("Validation errors:\n{}", errors.join("\n"));
        log::error!("{}", error_msg);
        return Err(ServerFnError::new(error_msg));
    }

    // Validate all students
    for student in &students {
        if let Err(validation_errors) = student.validate() {
            let error_msg = format!(
                "Validation failed for student {} {}: {:?}",
                student.firstname, student.lastname, validation_errors
            );
            log::error!("{}", error_msg);
            return Err(ServerFnError::new(error_msg));
        }
    }

    Ok(students)
}

fn parse_student_record(
    record: StudentCsvRow,
    row_num: usize,
) -> Result<AddStudentRequest, String> {
    let gender = GenderEnum::from_str(&record.gender)
        .map_err(|e| format!("Row {}: Invalid gender '{}': {}", row_num, record.gender, e))?;

    let date_of_birth =
        NaiveDate::parse_from_str(&record.date_of_birth, "%Y-%m-%d").map_err(|e| {
            format!(
                "Row {}: Invalid date of birth '{}': {}",
                row_num, record.date_of_birth, e
            )
        })?;

    let current_grade_level = GradeEnum::from_str(&record.current_grade_level).map_err(|e| {
        format!(
            "Row {}: Invalid grade '{}': {}",
            row_num, record.current_grade_level, e
        )
    })?;

    let intervention = if record.intervention == "None" || record.intervention.trim().is_empty() {
        None
    } else {
        Some(
            InterventionEnum::from_str(&record.intervention).map_err(|e| {
                format!(
                    "Row {}: Invalid intervention '{}': {}",
                    row_num, record.intervention, e
                )
            })?,
        )
    };

    let esl = ESLEnum::from_str(&record.esl)
        .map_err(|e| format!("Row {}: Invalid ESL value '{}': {}", row_num, record.esl, e))?;

    Ok(AddStudentRequest {
        firstname: record.firstname.trim().to_string(),
        lastname: record.lastname.trim().to_string(),
        preferred: record.preferred.trim().to_string(),
        gender,
        date_of_birth,
        student_id: record.student_id,
        esl,
        current_grade_level,
        teacher: record.teacher.trim().to_string(),
        iep: record.iep,
        bip: record.bip,
        student_504: record.student_504,
        readplan: record.readplan,
        gt: record.gt,
        intervention,
        eye_glasses: record.eye_glasses,
        notes: record.notes.trim().to_string(),
        pin: record.pin,
    })
}
/*
#[server(BulkStudentEnrollment, "/api")]
pub async fn bulk_student_enrollment(file_contents: String) -> Result<usize, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Use csv crate to parse the file
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file_contents.as_bytes());

        let mut imported_enrollments = Vec::new();

        for result in rdr.deserialize::<StudentCsvRow>() {
            match result {
                Ok(record) => {}
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError(
            "Server-side functionality not available".to_string(),
        ))
    }
}*/
