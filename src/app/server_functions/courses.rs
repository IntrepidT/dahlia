use crate::app::models::course::{Course, CreateCourseRequest, UpdateCourseRequest};
use leptos::*;

#[cfg(feature = "ssr")]
use {crate::app::db::course_database, actix_web::web, sqlx::PgPool, std::error::Error};

#[server(GetCourses, "/api")]
pub async fn get_courses() -> Result<Vec<Course>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract database pool: {}", e)))?;

        log::info!("Attempting to retrieve all courses");

        match course_database::get_all_courses(&pool).await {
            Ok(courses) => {
                log::info!("Successfully retrieved {} courses", courses.len());
                Ok(courses)
            }
            Err(e) => {
                log::error!("Error retrieving courses: {}", e);
                Err(ServerFnError::ServerError(
                    "Failed to retrieve courses".into(),
                ))
            }
        }
    }
}

#[server(GetCourse, "/api")]
pub async fn get_course(course_id: i32) -> Result<Course, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract database pool: {}", e)))?;

        log::info!("Attempting to retrieve course with ID: {}", course_id);

        match course_database::get_course_by_id(&pool, course_id).await {
            Ok(course) => {
                log::info!("Successfully retrieved course with ID: {}", course_id);
                Ok(course)
            }
            Err(e) => {
                log::error!("Error retrieving course with ID {}: {}", course_id, e);
                Err(ServerFnError::ServerError(
                    "Failed to retrieve course".into(),
                ))
            }
        }
    }
}

#[server(GetCourseByCode, "/api")]
pub async fn get_course_by_code(course_code: String) -> Result<Course, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract database pool: {}", e)))?;

        log::info!("Attempting to retrieve course with code: {}", course_code);

        match course_database::get_course_by_code(&pool, &course_code).await {
            Ok(course) => {
                log::info!("Successfully retrieved course with code: {}", course_code);
                Ok(course)
            }
            Err(e) => {
                log::error!("Error retrieving course with code {}: {}", course_code, e);
                Err(ServerFnError::ServerError(
                    "Failed to retrieve course".into(),
                ))
            }
        }
    }
}

#[server(AddCourse, "/api")]
pub async fn add_course(add_course_request: CreateCourseRequest) -> Result<Course, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract database pool: {}", e)))?;

        log::info!(
            "Attempting to add a new course with code: {}",
            add_course_request.course_code
        );

        match course_database::add_course(&pool, add_course_request).await {
            Ok(course) => {
                log::info!(
                    "Successfully added course with code: {}",
                    course.course_code
                );
                Ok(course)
            }
            Err(e) => {
                log::error!("Error adding course: {}", e);
                Err(ServerFnError::ServerError("Failed to add course".into()))
            }
        }
    }
}

#[server(UpdateCourse, "/api")]
pub async fn update_course(
    course_id: i32,
    update_course_request: UpdateCourseRequest,
) -> Result<Course, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract database pool: {}", e)))?;

        log::info!("Attempting to update course with ID: {}", course_id);

        match course_database::update_course(&pool, course_id, update_course_request).await {
            Ok(course) => {
                log::info!("Successfully updated course with ID: {}", course_id);
                Ok(course)
            }
            Err(e) => {
                log::error!("Error updating course with ID {}: {}", course_id, e);
                Err(ServerFnError::ServerError("Failed to update course".into()))
            }
        }
    }
}

#[server(DeleteCourse, "/api")]
pub async fn delete_course(course_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract database pool: {}", e)))?;

        log::info!("Attempting to delete course with ID: {}", course_id);

        match course_database::delete_course(&pool, course_id).await {
            Ok(_) => {
                log::info!("Successfully deleted course with ID: {}", course_id);
                Ok(())
            }
            Err(e) => {
                log::error!("Error deleting course with ID {}: {}", course_id, e);
                Err(ServerFnError::ServerError("Failed to delete course".into()))
            }
        }
    }
}
