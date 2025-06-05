use crate::app::models::enrollment::{
    AcademicYear, CreateEnrollmentRequest, Enrollment, EnrollmentStatus, UpdateEnrollmentRequest,
};
use leptos::*;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use {crate::app::db::enrollment_database, actix_web::web, sqlx::PgPool, std::error::Error};

#[server(GetEnrollments, "/api")]
pub async fn get_enrollments() -> Result<Vec<Enrollment>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve all enrollments from the database"); // Fixed log message

        match enrollment_database::get_all_enrollments(&pool).await {
            Ok(enrollments) => {
                log::info!("Successfully retrieved enrollments: {:?}", enrollments);
                Ok(enrollments)
            }
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to retrieve enrollments: {}",
                e
            ))),
        }
    }
}

#[server(GetEnrollment, "/api")]
pub async fn get_enrollment(
    student_id: i32,
    academic_year: AcademicYear,
) -> Result<Enrollment, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!(
            "Attempting to retrieve enrollment for student_id: {} and academic_year: {:?}",
            student_id,
            academic_year
        );

        match enrollment_database::get_enrollment_by_student_and_year(
            &pool,
            student_id,
            academic_year,
        )
        .await
        {
            Ok(enrollment) => {
                log::info!("Successfully retrieved enrollment: {:?}", enrollment);
                Ok(enrollment)
            }
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to retrieve enrollment: {}",
                e
            ))),
        }
    }
}

#[server(GetEnrollmentsByStudent, "/api")]
pub async fn get_enrollments_by_student(student_id: i32) -> Result<Vec<Enrollment>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!(
            "Attempting to retrieve enrollments for student_id: {}",
            student_id
        );

        match enrollment_database::get_enrollments_by_student(&student_id, &pool).await {
            Ok(enrollments) => {
                log::info!(
                    "Successfully retrieved enrollments for student: {:?}",
                    enrollments
                );
                Ok(enrollments)
            }
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to retrieve enrollments for student: {}",
                e
            ))),
        }
    }
}

#[server(GetEnrollmentByYear, "/api")]
pub async fn get_enrollments_by_year(
    academic_year: AcademicYear,
) -> Result<Vec<Enrollment>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!(
            "Attempting to retrieve enrollments for academic_year: {:?}",
            academic_year
        );

        match enrollment_database::get_enrollments_by_academic_year(&academic_year, &pool).await {
            Ok(enrollments) => {
                log::info!(
                    "Successfully retrieved enrollments for year: {:?}",
                    enrollments
                );
                Ok(enrollments)
            }
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to retrieve enrollments for year: {}",
                e
            ))),
        }
    }
}

#[server(GetEnrollmentsByTeacher, "/api")]
pub async fn get_enrollments_by_teacher(teacher_id: i32) -> Result<Vec<Enrollment>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!(
            "Attempting to retrieve enrollments for teacher_id: {}",
            teacher_id
        );

        match enrollment_database::get_enrollments_by_teacher(teacher_id, &pool).await {
            Ok(enrollments) => {
                log::info!(
                    "Successfully retrieved enrollments for teacher: {:?}",
                    enrollments
                );
                Ok(enrollments)
            }
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to retrieve enrollments for teacher: {}",
                e
            ))),
        }
    }
}

#[server(CreateEnrollment, "/api")]
pub async fn create_enrollment(
    new_enrollment_request: CreateEnrollmentRequest,
) -> Result<Enrollment, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!(
            "Attempting to create enrollment for student_id: {}, academic_year: {:?}",
            new_enrollment_request.student_id,
            new_enrollment_request.academic_year
        );

        // Convert CreateEnrollmentRequest to Enrollment
        let enrollment = Enrollment::new(
            new_enrollment_request.student_id,
            new_enrollment_request.academic_year,
            new_enrollment_request.grade_level,
            new_enrollment_request.teacher_id,
            new_enrollment_request.status,
            new_enrollment_request.enrollment_date,
            new_enrollment_request.status_change_date, // Now Option<NaiveDate>
            new_enrollment_request.notes,
        );

        match enrollment_database::add_enrollment(&enrollment, &pool).await {
            Ok(enrollment) => {
                log::info!("Successfully created enrollment: {:?}", enrollment);
                Ok(enrollment)
            }
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to create enrollment: {}",
                e
            ))),
        }
    }
}

#[server(ModifyEnrollment, "/api")]
pub async fn modify_enrollment(
    update_enrollment_request: UpdateEnrollmentRequest,
) -> Result<Enrollment, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!(
            "Attempting to modify enrollment for student_id: {}, academic_year: {:?}",
            update_enrollment_request.student_id,
            update_enrollment_request.academic_year
        );

        match enrollment_database::update_enrollment(
            &update_enrollment_request.student_id,
            &update_enrollment_request.academic_year,
            update_enrollment_request.grade_level,
            update_enrollment_request.teacher_id,
            update_enrollment_request.status,
            update_enrollment_request.enrollment_date,
            update_enrollment_request.status_change_date, // Now Option<NaiveDate>
            update_enrollment_request.notes,
            &pool,
        )
        .await
        {
            Ok(Some(enrollment)) => {
                log::info!("Successfully modified enrollment: {:?}", enrollment);
                Ok(enrollment)
            }
            Ok(None) => Err(ServerFnError::new("Enrollment not found".to_string())),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to modify enrollment: {}",
                e
            ))),
        }
    }
}

#[server(UpdateEnrollmentStatus, "/api")]
pub async fn update_enrollment_status(
    student_id: i32,
    academic_year: AcademicYear,
    status: EnrollmentStatus,
) -> Result<Enrollment, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use chrono::Utc;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to update enrollment status for student_id: {}, academic_year: {:?}, status: {:?}", 
            student_id, academic_year, status);

        let status_change_date = Utc::now().naive_utc().date();

        match enrollment_database::update_enrollment_status(
            &student_id,
            &academic_year,
            status,
            status_change_date,
            &pool,
        )
        .await
        {
            Ok(Some(enrollment)) => {
                log::info!("Successfully updated enrollment status: {:?}", enrollment);
                Ok(enrollment)
            }
            Ok(None) => Err(ServerFnError::new("Enrollment not found".to_string())),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update enrollment status: {}",
                e
            ))),
        }
    }
}

#[server(DeleteEnrollment, "/api")]
pub async fn delete_enrollment(
    student_id: i32,
    academic_year: AcademicYear,
) -> Result<Enrollment, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!(
            "Attempting to delete enrollment for student_id: {}, academic_year: {:?}",
            student_id,
            academic_year
        );

        match enrollment_database::delete_enrollment(student_id, academic_year, &pool).await {
            Ok(Some(enrollment)) => {
                log::info!("Successfully deleted enrollment: {:?}", enrollment);
                Ok(enrollment)
            }
            Ok(None) => Err(ServerFnError::new("Enrollment not found".to_string())),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to delete enrollment: {}",
                e
            ))),
        }
    }
}
