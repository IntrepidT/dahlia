use crate::app::models::bulk_student::StudentCsvRow;
use crate::app::models::student::{ESLEnum, GenderEnum, GradeEnum, Student};
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

        // Use csv crate to parse the file
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file_contents.as_bytes());

        let mut imported_students = Vec::new();

        for result in rdr.deserialize::<StudentCsvRow>() {
            match result {
                Ok(record) => {
                    // Add more robust parsing with error handling
                    let gender = match GenderEnum::from_str(&record.gender) {
                        Ok(g) => g,
                        Err(e) => {
                            log::error!("Invalid gender for student {}: {}", record.firstname, e);
                            return Err(ServerFnError::new(format!(
                                "Invalid gender for student {}",
                                record.firstname
                            )));
                        }
                    };

                    let date_of_birth =
                        match NaiveDate::parse_from_str(&record.date_of_birth, "%Y-%m-%d") {
                            Ok(date) => date,
                            Err(e) => {
                                log::error!(
                                    "Invalid date of birth for student {}: {}",
                                    record.firstname,
                                    e
                                );
                                return Err(ServerFnError::new(format!(
                                    "Invalid date of birth for student {}",
                                    record.firstname
                                )));
                            }
                        };

                    let grade = match GradeEnum::from_str(&record.grade) {
                        Ok(g) => g,
                        Err(e) => {
                            log::error!("Invalid grade for student {}: {}", record.firstname, e);
                            return Err(ServerFnError::new(format!(
                                "Invalid grade for student {}",
                                record.firstname
                            )));
                        }
                    };

                    let student = Student {
                        firstname: record.firstname,
                        lastname: record.lastname,
                        preferred: record.preferred,
                        gender,
                        date_of_birth,
                        student_id: record.student_id,
                        esl: ESLEnum::from_str(&record.esl).unwrap_or(ESLEnum::NotApplicable),
                        grade,
                        teacher: record.teacher,
                        iep: record.iep,
                        bip: record.bip,
                        student_504: record.student_504,
                        readplan: record.readplan,
                        gt: record.gt,
                        intervention: record.intervention,
                        eye_glasses: record.eye_glasses,
                        notes: record.notes,
                    };
                    imported_students.push(student);
                }
                Err(e) => {
                    log::error!("Error parsing CSV row: {:?}", e);
                    return Err(ServerFnError::new(format!("CSV parsing error: {}", e)));
                }
            }
        }

        // Validate students before bulk insert
        for student in &imported_students {
            if let Err(validation_errors) = student.validate() {
                let error_msg = format!(
                    "Validation failed for student {} {}: {:?}",
                    student.firstname, student.lastname, validation_errors
                );
                log::error!("{}", error_msg);
                return Err(ServerFnError::new(error_msg));
            }
        }

        // Bulk insert students
        match student_database::bulk_insert_students(imported_students, &pool).await {
            Ok(inserted_students) => Ok(inserted_students.len()),
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
