use crate::app::models::bulk_enrollment::EnrollmentCsvRow;
use crate::app::models::enrollment::{AcademicYear, Enrollment, EnrollmentStatus};
use crate::app::models::student::GradeEnum;
use chrono::{NaiveDate, Utc};
use csv::ReaderBuilder;
use leptos::prelude::*;
use std::str::FromStr;
use validator::Validate;

#[cfg(feature = "ssr")]
use {crate::app::db::enrollment_database, sqlx::PgPool};

#[server]
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
            Ok(count) => {
                log::info!("Successfully imported {} enrollments", count);
                Ok(count)
            }
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
        .flexible(true) // Allow varying number of fields
        .trim(csv::Trim::All) // Trim whitespace from all fields
        .from_reader(file_contents.as_bytes());

    let mut enrollments = Vec::new();
    let mut errors = Vec::new();

    // Get current date for defaults
    let current_date = Utc::now().date_naive();

    // Log headers for debugging
    let headers = rdr
        .headers()
        .map_err(|e| ServerFnError::new(format!("Failed to read CSV headers: {}", e)))?;
    log::info!("Enrollment CSV Headers: {:?}", headers);

    for (row_num, result) in rdr.records().enumerate() {
        match result {
            Ok(record) => {
                match record.deserialize::<EnrollmentCsvRow>(None) {
                    Ok(csv_row) => {
                        match parse_enrollment_record(csv_row, row_num + 2, current_date) {
                            // +2 for header and 1-based indexing
                            Ok(enrollment) => enrollments.push(enrollment),
                            Err(e) => errors.push(e),
                        }
                    }
                    Err(e) => {
                        errors.push(format!(
                            "Row {}: Failed to deserialize CSV row: {}",
                            row_num + 2,
                            e
                        ));
                    }
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
    for (index, enrollment) in enrollments.iter().enumerate() {
        if let Err(validation_errors) = enrollment.validate() {
            let error_msg = format!(
                "Validation failed for enrollment student_id {} (row {}): {:?}",
                enrollment.student_id,
                index + 2,
                validation_errors
            );
            log::error!("{}", error_msg);
            return Err(ServerFnError::new(error_msg));
        }
    }

    log::info!(
        "Successfully parsed and validated {} enrollments",
        enrollments.len()
    );
    Ok(enrollments)
}

fn parse_enrollment_record(
    record: EnrollmentCsvRow,
    row_num: usize,
    current_date: NaiveDate,
) -> Result<Enrollment, String> {
    // Parse and validate academic year
    let academic_year = AcademicYear::from_str(&record.academic_year.trim()).map_err(|e| {
        format!(
            "Row {}: Invalid academic year '{}': {}. Expected format like '2024-2025'",
            row_num, record.academic_year, e
        )
    })?;

    // Parse and validate grade level
    let grade_level = GradeEnum::from_str(&record.grade_level.trim()).map_err(|e| {
        format!(
            "Row {}: Invalid grade level '{}': {}. Valid values: Kindergarten, 1st Grade, 2nd Grade, etc.",
            row_num, record.grade_level, e
        )
    })?;

    // Validate student_id
    if record.student_id < 0 || record.student_id > 2000000000 {
        return Err(format!(
            "Row {}: Student ID {} is out of valid range (0-2000000000)",
            row_num, record.student_id
        ));
    }

    // Validate teacher_id - IMPORTANT: This uses teacher_id (integer) not teacher name
    // This reflects the decoupled architecture where enrollments use formal teacher IDs
    if record.teacher_id < 1 {
        return Err(format!(
            "Row {}: Teacher ID {} is invalid (must be positive integer)",
            row_num, record.teacher_id
        ));
    }

    // For bulk uploads, status is always "Active" - ignore CSV status field
    let status = EnrollmentStatus::Active;

    // For bulk uploads, enrollment_date is always current date - ignore CSV field
    let enrollment_date = current_date;

    // For bulk uploads, status_change_date defaults to current date - ignore CSV field
    let status_change_date = Some(current_date);

    // Handle notes field
    let notes = if record.notes.trim().is_empty() {
        None
    } else {
        Some(record.notes.trim().to_string())
    };

    Ok(Enrollment {
        student_id: record.student_id,
        academic_year,
        grade_level,
        teacher_id: record.teacher_id, // Using teacher_id (integer) for formal enrollment
        status,
        enrollment_date,
        status_change_date,
        notes,
    })
}

// Helper function to validate that a teacher_id exists in the teachers table
#[cfg(feature = "ssr")]
pub async fn validate_teacher_exists(
    teacher_id: i32,
    pool: &sqlx::PgPool,
) -> Result<bool, ServerFnError> {
    use sqlx::Row;

    let row = sqlx::query(
        "SELECT EXISTS(SELECT 1 FROM employees WHERE id = $1 AND role LIKE '%Teacher%')",
    )
    .bind(teacher_id)
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error checking teacher: {}", e)))?;

    let exists: bool = row.get("exists");
    Ok(exists)
}

// Enhanced version with teacher validation
#[server]
pub async fn upload_bulk_enrollment_with_validation(
    file_contents: String,
) -> Result<usize, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Parse and validate all rows in the CSV file
        let enrollments = parse_and_validate_enrollments(&file_contents)?;

        // Additional validation: Check that all teacher_ids exist
        let mut validation_errors = Vec::new();
        for enrollment in &enrollments {
            match validate_teacher_exists(enrollment.teacher_id, &pool).await {
                Ok(exists) => {
                    if !exists {
                        validation_errors.push(format!(
                            "Teacher ID {} does not exist or is not a teacher",
                            enrollment.teacher_id
                        ));
                    }
                }
                Err(e) => {
                    validation_errors.push(format!(
                        "Failed to validate teacher ID {}: {}",
                        enrollment.teacher_id, e
                    ));
                }
            }
        }

        if !validation_errors.is_empty() {
            let error_msg = format!(
                "Teacher validation errors:\n{}",
                validation_errors.join("\n")
            );
            log::error!("{}", error_msg);
            return Err(ServerFnError::new(error_msg));
        }

        // Bulk insert using optimized method
        match enrollment_database::bulk_insert_enrollments(&pool, &enrollments).await {
            Ok(count) => {
                log::info!(
                    "Successfully imported {} enrollments with teacher validation",
                    count
                );
                Ok(count)
            }
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
