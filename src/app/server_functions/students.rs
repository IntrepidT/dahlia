use crate::app::errors::ErrorMessage;
use crate::app::models::{
    student::Student, AddStudentRequest, DeleteStudentRequest, UpdateStudentRequest,
};
use leptos::*;

#[cfg(feature = "ssr")]
use {
    crate::app::db::database, crate::app::db::student_database, crate::app::errors::StudentError,
    actix_web::web, chrono::Local, sqlx::PgPool, std::error::Error, uuid::Uuid,
};

#[server(GetStudents, "/api")]
pub async fn get_students() -> Result<Vec<Student>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve all students from database");

        match student_database::get_all_students(&pool).await {
            Ok(students) => {
                log::info!("Successfully retrieve_all_students from database");
                Ok(students)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(GetStudentsSmart, "/api")]
pub async fn get_students_smart(fragment: String) -> Result<Vec<Student>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve all students from database (smartly)");

        match student_database::get_all_students(&pool).await {
            Ok(students) => {
                log::info!("Successfully retrieve_all_students from database");
                Ok(students)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(AddStudent, "/api")]
pub async fn add_student(add_student_request: AddStudentRequest) -> Result<Student, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to add new student to the database");
        let bufferStudent = Student::new(
            add_student_request.firstname,
            add_student_request.lastname,
            add_student_request.preferred,
            add_student_request.gender,
            add_student_request.date_of_birth,
            add_student_request.student_id,
            add_student_request.esl,
            add_student_request.grade,
            add_student_request.teacher,
            add_student_request.iep,
            add_student_request.bip,
            add_student_request.student_504,
            add_student_request.readplan,
            add_student_request.gt,
            add_student_request.intervention,
            add_student_request.eye_glasses,
            add_student_request.notes,
        );

        match student_database::add_student(&bufferStudent, &pool).await {
            Ok(created_student) => {
                log::info!(
                    "Successfully created student with ID: {}",
                    created_student.student_id
                );
                Ok(created_student)
            }
            Err(e) => {
                log::info!("Failed to create student: {:?}", e);
                Err(ServerFnError::new(format!(
                    "The object created was not a student...somehow?"
                )))
            }
        }
    }
}

#[server(DeleteStudent, "/api")]
pub async fn delete_student(
    delete_student_request: DeleteStudentRequest,
) -> Result<Student, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to delete student to the database");

        match student_database::delete_student(
            delete_student_request.firstname,
            delete_student_request.lastname,
            delete_student_request.student_id,
            &pool,
        )
        .await
        {
            Ok(deleted) => Ok(deleted),
            Err(_) => Err(ServerFnError::new(
                "Failed to delete student from the database",
            )),
        }
    }
}

#[server(EditStudent, "/api")]
pub async fn edit_student(
    edit_student_request: UpdateStudentRequest,
) -> Result<Student, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to update student in the database");

        match student_database::update_student(
            edit_student_request.firstname,
            edit_student_request.lastname,
            edit_student_request.preferred,
            edit_student_request.gender,
            edit_student_request.date_of_birth,
            edit_student_request.student_id,
            edit_student_request.esl,
            edit_student_request.grade,
            edit_student_request.teacher,
            edit_student_request.iep,
            edit_student_request.bip,
            edit_student_request.student_504,
            edit_student_request.readplan,
            edit_student_request.gt,
            edit_student_request.intervention,
            edit_student_request.eye_glasses,
            edit_student_request.notes,
            &pool,
        )
        .await
        {
            Ok(Some(updated_student)) => Ok(updated_student),
            Ok(None) => Err(ServerFnError::new(format!(
                "An None Value was returned instead of an updated student"
            ))),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update student: {}",
                e
            ))),
        }
    }
}
