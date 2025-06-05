use crate::app::models::bulk_enrollment::EnrollmentCsvRow;
use crate::app::models::enrollment::{AcademicYear, Enrollment, EnrollmentStatus};
use crate::app::models::student::GradeEnum;
use chrono::{NaiveDate, Utc};
use csv::ReaderBuilder;
use leptos::*;
use std::str::FromStr;
use validator::Validate;

#[cfg(feature = "ssr")]
use {crate::app::db::enrollment_database, sqlx::PgPool};

#[server(UploadBulkEnrollment, "/api")]
pub async fn upload_bulk_enrollment(file_contents: String) -> Result<usize, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Parse and validate all rows in the CSV file
        let enrollments = parse_and_validate_enrollments(&file_contents)?;

        // Bulk insert using optimized method
        match enrollment_database::bulk_insert_enrollments(&pool, &enrollments).await {
            Ok(count) => Ok(count),
            Err(e) => {
                log::error!("Failed to insert enrollments: {}", e);
                Err(ServerFnError::ServerError(format!("Import failed: {}", e)))
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError(
            "Server-side rendering is not enabled".to_string(),
        ))
    }
}

// Separate parsing logic for reusability and better error handling
fn parse_and_validate_enrollments(file_contents: &str) -> Result<Vec<Enrollment>, ServerFnError> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file_contents.as_bytes());

    let mut enrollments = Vec::new();
    let mut errors = Vec::new();

    // Get current date for defaults
    let current_date = Utc::now().date_naive();

    for (row_num, result) in rdr.deserialize::<EnrollmentCsvRow>().enumerate() {
        match result {
            Ok(record) => {
                match parse_enrollment_record(record, row_num + 2, current_date) {
                    // +2 for header and 1-based indexing
                    Ok(enrollment) => enrollments.push(enrollment),
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

    // Validate all enrollments
    for enrollment in &enrollments {
        if let Err(validation_errors) = enrollment.validate() {
            let error_msg = format!(
                "Validation failed for enrollment student_id {}: {:?}",
                enrollment.student_id, validation_errors
            );
            log::error!("{}", error_msg);
            return Err(ServerFnError::new(error_msg));
        }
    }

    Ok(enrollments)
}

fn parse_enrollment_record(
    record: EnrollmentCsvRow,
    row_num: usize,
    current_date: NaiveDate,
) -> Result<Enrollment, String> {
    let academic_year = AcademicYear::from_str(&record.academic_year).map_err(|e| {
        format!(
            "Row {}: Invalid academic year '{}': {}",
            row_num, record.academic_year, e
        )
    })?;

    let grade_level = GradeEnum::from_str(&record.grade_level).map_err(|e| {
        format!(
            "Row {}: Invalid grade level '{}': {}",
            row_num, record.grade_level, e
        )
    })?;

    // For bulk uploads, status is always "Active" - ignore CSV status field
    let status = EnrollmentStatus::Active;

    // For bulk uploads, enrollment_date is always current date - ignore CSV field
    let enrollment_date = current_date;

    // For bulk uploads, status_change_date defaults to current date - ignore CSV field
    let status_change_date = Some(current_date);

    let notes = if record.notes.trim().is_empty() {
        None
    } else {
        Some(record.notes.trim().to_string())
    };

    Ok(Enrollment {
        student_id: record.student_id,
        academic_year,
        grade_level,
        teacher_id: record.teacher_id,
        status,
        enrollment_date,
        status_change_date,
        notes,
    })
}
