use crate::app::models::bulk_student::{StudentCsvRow, StudentCsvRowAlternative};
use crate::app::models::student::{
    AddStudentRequest, ESLEnum, GenderEnum, GradeEnum, InterventionEnum,
};
use chrono::NaiveDate;
use csv::ReaderBuilder;
use leptos::prelude::*;
use std::str::FromStr;
use validator::Validate;

#[cfg(feature = "ssr")]
use {crate::app::db::student_database, sqlx::PgPool};

#[server]
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

// Enhanced parsing logic with better error handling and flexibility
fn parse_and_validate_students(
    file_contents: &str,
) -> Result<Vec<AddStudentRequest>, ServerFnError> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true) // Allow varying number of fields
        .trim(csv::Trim::All) // Trim whitespace from all fields
        .from_reader(file_contents.as_bytes());

    let mut students = Vec::new();
    let mut errors = Vec::new();

    // First, try to get headers to determine the format
    let headers = rdr
        .headers()
        .map_err(|e| ServerFnError::new(format!("Failed to read CSV headers: {}", e)))?;

    log::info!("CSV Headers: {:?}", headers);

    // Check if we have 'grade' or 'current_grade_level' in headers
    let has_grade_field = headers.iter().any(|h| h == "grade");
    let has_current_grade_level_field = headers.iter().any(|h| h == "current_grade_level");

    for (row_num, result) in rdr.records().enumerate() {
        match result {
            Ok(record) => {
                // Try to parse based on detected format
                let student_result = if has_grade_field && !has_current_grade_level_field {
                    // Use alternative format (matching CSV template with 'grade' header)
                    match record.deserialize::<StudentCsvRowAlternative>(None) {
                        Ok(alt_record) => {
                            let converted_record = StudentCsvRow::from(alt_record);
                            parse_student_record(converted_record, row_num + 2)
                        }
                        Err(e) => Err(format!(
                            "Row {}: Failed to parse CSV row: {}",
                            row_num + 2,
                            e
                        )),
                    }
                } else {
                    // Use standard format
                    match record.deserialize::<StudentCsvRow>(None) {
                        Ok(std_record) => parse_student_record(std_record, row_num + 2),
                        Err(e) => Err(format!(
                            "Row {}: Failed to parse CSV row: {}",
                            row_num + 2,
                            e
                        )),
                    }
                };

                match student_result {
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
    for (index, student) in students.iter().enumerate() {
        if let Err(validation_errors) = student.validate() {
            let error_msg = format!(
                "Validation failed for student {} {} (row {}): {:?}",
                student.firstname,
                student.lastname,
                index + 2,
                validation_errors
            );
            log::error!("{}", error_msg);
            return Err(ServerFnError::new(error_msg));
        }
    }

    log::info!(
        "Successfully parsed and validated {} students",
        students.len()
    );
    Ok(students)
}

fn parse_student_record(
    record: StudentCsvRow,
    row_num: usize,
) -> Result<AddStudentRequest, String> {
    // Parse and validate gender
    let gender = GenderEnum::from_str(&record.gender.trim()).map_err(|e| {
        format!(
            "Row {}: Invalid gender '{}': {}. Valid values: Male, Female, Non-binary",
            row_num, record.gender, e
        )
    })?;

    // Parse and validate date of birth
    let date_of_birth = NaiveDate::parse_from_str(&record.date_of_birth.trim(), "%Y-%m-%d")
        .map_err(|e| {
            format!(
                "Row {}: Invalid date of birth '{}': {}. Expected format: YYYY-MM-DD",
                row_num, record.date_of_birth, e
            )
        })?;

    // Parse and validate current grade level
    let current_grade_level = GradeEnum::from_str(&record.current_grade_level.trim())
        .map_err(|e| {
            format!(
                "Row {}: Invalid grade '{}': {}. Valid values: Kindergarten, 1st Grade, 2nd Grade, etc.",
                row_num, record.current_grade_level, e
            )
        })?;

    // Parse and validate intervention (optional field)
    let intervention = if record.intervention.trim() == "None"
        || record.intervention.trim().is_empty()
    {
        None
    } else {
        Some(
            InterventionEnum::from_str(&record.intervention.trim()).map_err(|e| {
                format!(
                    "Row {}: Invalid intervention '{}': {}. Valid values: Literacy, Math, Literacy and Math, None",
                    row_num, record.intervention, e
                )
            })?,
        )
    };

    // Parse and validate ESL
    let esl = ESLEnum::from_str(&record.esl.trim())
        .map_err(|e| format!("Row {}: Invalid ESL value '{}': {}. Valid values: Not Applicable, Spanish, Arabic, etc.", 
                            row_num, record.esl, e))?;

    // Validate student ID range
    if record.student_id < 0 || record.student_id > 2000000000 {
        return Err(format!(
            "Row {}: Student ID {} is out of valid range (0-2000000000)",
            row_num, record.student_id
        ));
    }

    // Validate PIN range (assuming similar constraints)
    if record.pin < 0 || record.pin > 99999999 {
        return Err(format!(
            "Row {}: PIN {} is out of valid range",
            row_num, record.pin
        ));
    }

    // Validate required string fields are not empty
    if record.firstname.trim().is_empty() {
        return Err(format!("Row {}: First name cannot be empty", row_num));
    }
    if record.lastname.trim().is_empty() {
        return Err(format!("Row {}: Last name cannot be empty", row_num));
    }

    // IMPORTANT: Teacher field validation for decoupled model
    // Since teachers are decoupled, we now store teacher names as strings
    if record.teacher.trim().is_empty() {
        return Err(format!("Row {}: Teacher name cannot be empty", row_num));
    }

    // Validate teacher name is reasonable (basic validation)
    let teacher_name = record.teacher.trim();
    if teacher_name.len() < 2 {
        return Err(format!(
            "Row {}: Teacher name '{}' is too short",
            row_num, teacher_name
        ));
    }
    if teacher_name.len() > 100 {
        return Err(format!(
            "Row {}: Teacher name '{}' is too long (max 100 characters)",
            row_num, teacher_name
        ));
    }

    Ok(AddStudentRequest {
        firstname: record.firstname.trim().to_string(),
        lastname: record.lastname.trim().to_string(),
        preferred: record.preferred.trim().to_string(),
        gender,
        date_of_birth,
        student_id: record.student_id,
        esl,
        current_grade_level,
        teacher: teacher_name.to_string(), // Store as string, not ID
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
