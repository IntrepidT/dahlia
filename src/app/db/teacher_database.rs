//this is a filler comment for now
cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {

        use crate::app::models::student::GradeEnum;
        use crate::app::models::employee::{Employee, EmployeeRole, StatusEnum};
        use log::{debug, error, info, warn};
        use leptos::*;
        use sqlx::prelude::*;
        use sqlx::PgPool;
        use tokio::*;

        pub async fn get_all_teachers(pool: &sqlx::PgPool) -> Result<Vec<Employee>, ServerFnError> {
            let rows = sqlx::query(
                "SELECT id, firstname, lastname, status, grade
                 FROM employees 
                 WHERE role = 'Teacher'::employee_role
                 ORDER BY lastname DESC"
            )
            .fetch_all(pool)
            .await?;

            let teachers: Vec<Employee> = rows
                .into_iter()
                .map(|row| {
                    let id: i32 = row.get("id");
                    let firstname: String = row.get("firstname");
                    let lastname: String = row.get("lastname");
                    let status: StatusEnum = row.get("status");
                    let grade: Option<GradeEnum> = row.get("grade");

                    Employee::new_teacher(
                        id,
                        firstname,
                        lastname,
                        status,
                        grade,
                    )
                })
                .collect();
            Ok(teachers)
        }

        pub async fn add_teacher(buffer_teacher: &Employee, pool: &PgPool) -> Result<Employee, ServerFnError> {
            // Verify that we're actually adding a teacher
            if let EmployeeRole::Teacher { grade } = &buffer_teacher.role {
                let row = sqlx::query(
                    "INSERT INTO employees (id, firstname, lastname, status, role, grade)
                     VALUES ($1, $2, $3, $4, 'Teacher'::employee_role, $5) 
                     RETURNING id, firstname, lastname, status, grade"
                )
                    .bind(&buffer_teacher.id)
                    .bind(&buffer_teacher.firstname)
                    .bind(&buffer_teacher.lastname)
                    .bind(&buffer_teacher.status)
                    .bind(grade)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

                let teacher = Employee::new_teacher(
                    row.get("id"),
                    row.get("firstname"),
                    row.get("lastname"),
                    row.get("status"),
                    row.get("grade"),
                );

                Ok(teacher)
            } else {
                Err(ServerFnError::new("Attempted to add non-teacher employee using add_teacher"))
            }
        }

        pub async fn delete_teacher(id: i32, pool: &PgPool) -> Result<Employee, ServerFnError> {
            let row = sqlx::query(
                "DELETE FROM employees
                 WHERE id = $1 AND role = 'Teacher'::employee_role 
                 RETURNING id, firstname, lastname, status, grade"
            )
                .bind(id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let teacher = Employee::new_teacher(
                row.get("id"),
                row.get("firstname"),
                row.get("lastname"),
                row.get("status"),
                row.get("grade"),
            );

            Ok(teacher)
        }

        pub async fn update_teacher(
            id: i32,
            firstname: String,
            lastname: String,
            status: StatusEnum,
            grade: Option<GradeEnum>,
            pool: &PgPool
        ) -> Result<Option<Employee>, ServerFnError> {
            let row = sqlx::query(
                "UPDATE employees
                 SET firstname = $1, lastname = $2, status = $3::status_enum, grade = $4
                 WHERE id = $5 AND role = 'Teacher'::employee_role 
                 RETURNING id, firstname, lastname, status, grade"
            )
                .bind(&firstname)
                .bind(&lastname)
                .bind(&status)
                .bind(&grade)
                .bind(&id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let teacher = Employee::new_teacher(
                row.get("id"),
                row.get("firstname"),
                row.get("lastname"),
                row.get("status"),
                row.get("grade"),
            );

            Ok(Some(teacher))
        }
    }
}
