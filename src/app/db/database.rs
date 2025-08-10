use crate::app::models::student::GradeEnum;
use crate::app::models::EmployeeRole;
use leptos::prelude::*;

cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")] {

        use log::{debug, error, info, warn};
        use crate::app::models::Employee;
        use crate::app::models::StatusEnum;
        use dotenvy::dotenv;
        use sqlx::{PgPool, Row};
        use std::env;
        use tokio::*;
        use std::sync::Once;

        // Use Once to ensure dotenv is only called once
        static INIT: Once = Once::new();

        fn init_env() {
            INIT.call_once(|| {
                match dotenv() {
                    Ok(_) => info!("Environment variables loaded from .env file"),
                    Err(e) => {
                        warn!("Could not load .env file: {}. Using system environment variables.", e);
                    }
                }
            });
        }

       pub async fn create_pool() -> sqlx::PgPool {
           // Initialize environment variables only once
           init_env();

            let database_url = env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment variables or .env file");

            info!("Creating database connection pool...");

            //create the connection pool using sqlx
            let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(&database_url.as_str())
            .await
            .expect("Failed to create PostgreSQL pool");

            info!("Running database migrations...");
            sqlx::migrate!()
                .run(&pool)
                .await
                .expect("migrations failed");

            info!("Database pool and migrations completed successfully");
           pool
        }

       pub async fn get_all_employees(pool: &sqlx::PgPool) -> Result<Vec<Employee>, ServerFnError> {
           let rows = sqlx::query("SELECT id, firstname, lastname, status, role, grade FROM employees")
               .fetch_all(pool)
               .await?;

           let employees: Vec<Employee> = rows
               .into_iter()
               .map(|row| {
                   let id: i32 = row.get("id");
                   let firstname: String = row.get("firstname");
                   let lastname: String = row.get("lastname");
                   let status: StatusEnum = row.get("status");
                   let role: EmployeeRole = row.get("role");
                   let grade: Option<GradeEnum> = row.get("grade");

                   let role = match role {
                       EmployeeRole::Teacher { grade: _ } => EmployeeRole::Teacher { grade },
                       other => other,
                   };

                   Employee {
                            id,
                            firstname,
                            lastname,
                            status,
                            role,
                   }
               }).collect();
           Ok(employees)
       }

        pub async fn add_employee(employee: &Employee, pool: &PgPool) -> Result<Employee, ServerFnError> {
            let grade = match &employee.role {
                EmployeeRole::Teacher { grade } => grade.clone(),
                _ => None
            };

            let row = match &employee.role {
                EmployeeRole::Teacher { grade } => {
                    sqlx::query(
                        "INSERT INTO employees (firstname, lastname, status, role, grade)
                         VALUES ($1, $2, $3, $4, $5) 
                         RETURNING id, firstname, lastname, status, role, grade"
                    )
                    .bind(&employee.firstname)
                    .bind(&employee.lastname)
                    .bind(&employee.status)
                    .bind(&employee.role)
                    .bind(&grade)
                    .fetch_one(pool)
                    .await
                },
                _ => {
                    sqlx::query(
                        "INSERT INTO employees (firstname, lastname, status, role)
                         VALUES ($1, $2, $3, $4) 
                         RETURNING id, firstname, lastname, status, role, grade"
                    )
                    .bind(&employee.firstname)
                    .bind(&employee.lastname)
                    .bind(&employee.status)
                    .bind(&employee.role)
                    .fetch_one(pool)
                    .await
                }
            }.map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            debug!("Raw database role: {:?}", row.get::<String, _>("role"));
            debug!("Raw database grade: {:?}", row.get::<Option<String>, _>("grade"));

            let id: i32 = row.get("id");
            let firstname: String = row.get("firstname");
            let lastname: String = row.get("lastname");
            let status: StatusEnum = row.get("status");
            let role: EmployeeRole = row.get("role");

            Ok(Employee::new(id, firstname, lastname, status, role))
        }

        pub async fn update_employee(employee: &Employee, pool: &PgPool) -> Result<Option<Employee>, ServerFnError> {
            info!("Starting employee update for ID: {}", employee.id);
            let grade = match &employee.role {
                EmployeeRole::Teacher { grade } => grade.clone(),
                _ => None
            };

            let row = match &employee.role {
                EmployeeRole::Teacher { grade } => {
                    sqlx::query(
                        "UPDATE employees
                         SET firstname = $1, lastname = $2, status = $3, 
                             role = $4, grade = $5
                         WHERE id = $6 
                         RETURNING id, firstname, lastname, status, role, grade"
                    )
                    .bind(&employee.firstname)
                    .bind(&employee.lastname)
                    .bind(&employee.status)
                    .bind(&employee.role)
                    .bind(&grade)
                    .bind(&employee.id)
                    .fetch_one(pool)
                    .await
                },
                _ => {
                    sqlx::query(
                        "UPDATE employees
                         SET firstname = $1, lastname = $2, status = $3::status_enum, 
                             role = $4::employee_role, grade = NULL
                         WHERE id = $5 
                         RETURNING id, firstname, lastname, status, role, grade"
                    )
                    .bind(&employee.firstname)
                    .bind(&employee.lastname)
                    .bind(&employee.status)
                    .bind(&employee.role)
                    .bind(&employee.id)
                    .fetch_one(pool)
                    .await
                }
            }.map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let id: i32 = row.get("id");
            let firstname: String = row.get("firstname");
            let lastname: String = row.get("lastname");
            let status: StatusEnum = row.get("status");
            let role: EmployeeRole = row.get("role");
            let grade: Option<GradeEnum> = row.get("grade");

            let role = match role {
               EmployeeRole::Teacher { grade: _ } => EmployeeRole::Teacher { grade },
               other => other,
            };
            log::info!("Successfully returned from database with updated employee");
           let updated_employee = Employee {
                    id,
                    firstname,
                    lastname,
                    status,
                    role,
           };

           Ok(Some(updated_employee))
        }

        pub async fn delete_employee(employee_id: i32, pool: &PgPool) -> Result<Employee, ServerFnError> {
            let row = sqlx::query(
                "DELETE FROM employees
                 WHERE id = $1
                 RETURNING id, firstname, lastname, status, role, grade"
            )
            .bind(employee_id)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            log::debug!("Deleted row: {:?}", row);

            let id: i32 = row.get("id");
            let firstname: String = row.get("firstname");
            let lastname: String = row.get("lastname");
            let status: StatusEnum = row.get("status");
            let role: EmployeeRole = row.get("role");
            let grade: Option<GradeEnum> = row.get("grade");

            // Fix the role for Teacher to include grade
            let role = match role {
                EmployeeRole::Teacher { grade: _ } => EmployeeRole::Teacher { grade },
                other => other,
            };

            Ok(Employee {
                id,
                firstname,
                lastname,
                status,
                role,
            })
        }
    }
}
