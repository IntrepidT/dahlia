//this file contains all handler functions for employees
use crate::app::db::teacher_database;
use crate::app::models::employee::Employee;
use crate::app::models::employee::{AddNewEmployeeRequest, UpdateEmployeeRequest};
use crate::app::models::DeleteTeacherRequest;
use crate::app::models::EmployeeRole;
use leptos::*;
use log::{error, info};

#[cfg(feature = "ssr")]
use {
    crate::app::db::database, actix_web::web, chrono::Local, sqlx::PgPool, std::error::Error,
    uuid::Uuid,
};

#[server(GetEmployees, "/api")]
pub async fn get_employees() -> Result<Vec<Employee>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve all teachers from database");
        use crate::app::db::database;

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

#[server(AddEmployee, "/api")]
pub async fn add_employee(
    add_employee_request: AddNewEmployeeRequest,
) -> Result<Employee, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to add new teacher to the database");

        let buffer_employee = match add_employee_request.role {
            EmployeeRole::Teacher { grade: _ } => Employee::new_teacher(
                0001,
                add_employee_request.firstname,
                add_employee_request.lastname,
                add_employee_request.status,
                add_employee_request.grade,
            ),
            EmployeeRole::AssistantPrincipal
            | EmployeeRole::Principal
            | EmployeeRole::Interventionist
            | EmployeeRole::IntegratedServices
            | EmployeeRole::Speech
            | EmployeeRole::OT
            | EmployeeRole::Psychologist
            | EmployeeRole::ParaProf
            | EmployeeRole::AssessmentCoordinator
            | EmployeeRole::Other => Employee::new(
                0001,
                add_employee_request.firstname,
                add_employee_request.lastname,
                add_employee_request.status,
                add_employee_request.role,
            ),
        };

        match database::add_employee(&buffer_employee, &pool).await {
            Ok(created_employee) => {
                log::info!(
                    "Successfully created employee: {} {}",
                    created_employee.firstname,
                    created_employee.lastname,
                );
                Ok(created_employee)
            }
            Err(e) => {
                log::info!("Failed to create employee: {:?}", e);
                Err(ServerFnError::new(format!(
                    "An error occured while creating the teacher"
                )))
            }
        }
    }
}

#[server(DeleteTeacher, "/api")]
pub async fn delete_employee(
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

        match database::delete_employee(
            delete_teacher_request.firstname,
            delete_teacher_request.lastname,
            &pool,
        )
        .await
        {
            Ok(deleted) => Ok(deleted),
            Err(_) => Err(ServerFnError::new(
                "Failed to delete teacher from the database",
            )),
        }
    }
}

#[server(EditTeacher, "/api")]
pub async fn edit_employee(
    update_employee_request: UpdateEmployeeRequest,
) -> Result<Employee, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to update teacher in the database");

        let buffer_employee = match update_employee_request.role {
            EmployeeRole::Teacher { grade: _ } => Employee::new_teacher(
                update_employee_request.id,
                update_employee_request.firstname,
                update_employee_request.lastname,
                update_employee_request.status,
                update_employee_request.grade,
            ),
            EmployeeRole::AssistantPrincipal
            | EmployeeRole::Principal
            | EmployeeRole::Interventionist
            | EmployeeRole::IntegratedServices
            | EmployeeRole::Speech
            | EmployeeRole::OT
            | EmployeeRole::Psychologist
            | EmployeeRole::ParaProf
            | EmployeeRole::AssessmentCoordinator
            | EmployeeRole::Other => Employee::new(
                update_employee_request.id,
                update_employee_request.firstname,
                update_employee_request.lastname,
                update_employee_request.status,
                update_employee_request.role,
            ),
        };

        match database::update_employee(&buffer_employee, &pool).await {
            Ok(Some(updated_employee)) => Ok(updated_employee),
            Ok(None) => Err(ServerFnError::new(format!(
                "Failed to update and return employee"
            ))),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update employee: {}",
                e
            ))),
        }
    }
}
