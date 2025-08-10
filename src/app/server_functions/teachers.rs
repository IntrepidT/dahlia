use crate::app::db::teacher_database;
use crate::app::models::employee::Employee;
use crate::app::models::teacher::{
    AddNewTeacherRequest, DeleteTeacherRequest, UpdateTeacherRequest,
};
use crate::app::models::StatusEnum;
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use {
    crate::app::db::database, actix_web::web, chrono::Local, sqlx::PgPool, std::error::Error,
    uuid::Uuid,
};

#[server]
pub async fn get_employees() -> Result<Vec<Employee>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve all teachers from database");

        match database::get_all_employees(&pool).await {
            Ok(employees) => {
                log::info!("Successfully retrieved all employees from database");
                Ok(employees)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server]
pub async fn get_teachers() -> Result<Vec<Employee>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve all teachers from database");

        match teacher_database::get_all_teachers(&pool).await {
            Ok(teachers) => {
                log::info!("Successfully retrieved all teachers from database");
                Ok(teachers)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server]
pub async fn add_teacher(
    add_teacher_request: AddNewTeacherRequest,
) -> Result<Employee, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to add new teacher to the database");
        let buffer_teacher = Employee::new_teacher(
            0001, //note that this value is simply a placeholder because the backend generates the
            //id automatically
            add_teacher_request.firstname,
            add_teacher_request.lastname,
            StatusEnum::NotApplicable,
            None,
        );

        match teacher_database::add_teacher(&buffer_teacher, &pool).await {
            Ok(created_teacher) => {
                log::info!(
                    "Successfully created teacher: {}{}",
                    created_teacher.firstname,
                    created_teacher.lastname,
                );
                Ok(created_teacher)
            }
            Err(e) => {
                log::info!("Failed to create teacher: {:?}", e);
                Err(ServerFnError::new(format!(
                    "An error occured while creating the teacher"
                )))
            }
        }
    }
}
/*
#[server]
pub async fn delete_teacher(
    delete_teacher_request: DeleteTeacherRequest,
) -> Result<Employee, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to delete teacher from the database");

        match teacher_database::delete_teacher(delete_teacher_request.id, &pool).await {
            Ok(deleted) => Ok(deleted),
            Err(_) => Err(ServerFnError::new(
                "Failed to delete teacher from the database",
            )),
        }
    }
}*/

#[server]
pub async fn edit_teacher(
    edit_teacher_request: UpdateTeacherRequest,
) -> Result<Employee, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to update teacher in the database");

        match teacher_database::update_teacher(
            edit_teacher_request.id,
            edit_teacher_request.firstname,
            edit_teacher_request.lastname,
            edit_teacher_request.status,
            edit_teacher_request.grade,
            &pool,
        )
        .await
        {
            Ok(Some(updated_student)) => Ok(updated_student),
            Ok(None) => Err(ServerFnError::new(format!(
                "Failed to update and return student"
            ))),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update student: {}",
                e
            ))),
        }
    }
}
