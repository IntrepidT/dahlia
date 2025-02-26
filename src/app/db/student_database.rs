cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")]{

        use crate::app::models::Student;
        use crate::app::models::student::{GradeEnum, ELLEnum, GenderEnum};
        use log::{debug, error, info, warn};
        use chrono::NaiveDate;
        use leptos::*;
        use sqlx::PgPool;
        use sqlx::prelude::*;

        pub async fn get_all_students(pool: &PgPool) -> Result<Vec<Student>, ServerFnError>{
            let rows  = sqlx::query("SELECT firstname, lastname, gender, date_of_birth, student_id, ell, grade, teacher, iep, student_504, readplan, gt, intervention, eye_glasses FROM students")
                .fetch_all(pool)
                .await?;

            let students: Vec<Student> = rows
                .into_iter()
                .map(|row| {
                    let firstname: String = row.get("firstname");
                    let lastname: String = row.get("lastname");
                    let gender: GenderEnum = row.get("gender");
                    let date_of_birth: chrono::NaiveDate = row.get::<NaiveDate, _>("date_of_birth");
                    let student_id: i32 = row.get("student_id");
                    let ell: ELLEnum = row.get("ell");
                    let grade: GradeEnum = row.get("grade");
                    let teacher: String = row.get("teacher");
                    let iep: bool = row.get("iep");
                    let student_504: bool = row.get("student_504");
                    let readplan: bool = row.get("readplan");
                    let gt: bool = row.get("gt");
                    let intervention: bool = row.get("intervention");
                    let eye_glasses: bool = row.get("eye_glasses");

                    Student {
                        firstname,
                        lastname,
                        gender,
                        date_of_birth,
                        student_id,
                        ell,
                        grade,
                        teacher,
                        iep,
                        student_504,
                        readplan,
                        gt,
                        intervention,
                        eye_glasses,
                    }
                })
                .collect();
            Ok(students)
        }

        pub async fn add_student(new_student: &Student, pool: &PgPool) -> Result<Student, ServerFnError>{
            let row = sqlx::query("INSERT INTO students (firstname, lastname, gender, date_of_birth, student_id, ell, grade, teacher, iep, student_504, readplan, gt, intervention, eye_glasses) VALUES($1, $2, $3::gender_enum, $4, $5, $6::ell_enum, $7::grade_enum, $8, $9, $10, $11, $12, $13, $14) RETURNING firstname, lastname, gender, date_of_birth, student_id, ell, grade, teacher, iep, student_504, readplan, gt, intervention, eye_glasses")
                .bind(&new_student.firstname)
                .bind(&new_student.lastname)
                .bind(&new_student.gender.to_string())
                .bind(&new_student.date_of_birth)
                .bind(&new_student.student_id)
                .bind(&new_student.ell.to_string())
                .bind(&new_student.grade.to_string())
                .bind(&new_student.teacher)
                .bind(&new_student.iep)
                .bind(&new_student.student_504)
                .bind(&new_student.readplan)
                .bind(&new_student.gt)
                .bind(&new_student.intervention)
                .bind(&new_student.eye_glasses)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let student = Student {
                firstname: row.get("firstname"),
                lastname: row.get("lastname"),
                gender: row.get("gender"),
                date_of_birth: row.get("date_of_birth"),
                student_id: row.get("student_id"),
                ell: row.get("ell"),
                grade: row.get("grade"),
                teacher: row.get("teacher"),
                iep: row.get("iep"),
                student_504: row.get("student_504"),
                readplan: row.get("readplan"),
                gt: row.get("gt"),
                intervention: row.get("intervention"),
                eye_glasses: row.get("eye_glasses"),
            };

            Ok(student)
        }

        pub async fn delete_student(firstname: String, lastname: String, student_id: i32, pool: &PgPool) -> Result<Student, ServerFnError> {
            let row = sqlx::query("DELETE FROM students WHERE firstname = $1 AND lastname = $2 AND student_id =$3 RETURNING firstname, lastname, gender, date_of_birth, student_id, ell, grade, teacher, iep, student_504, readplan, gt, intervention, eye_glasses")
                .bind(firstname)
                .bind(lastname)
                .bind(student_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let student = Student {
                firstname: row.get("firstname"),
                lastname: row.get("lastname"),
                gender: row.get("gender"),
                date_of_birth: row.get::<chrono::NaiveDate,_>("date_of_birth"),
                student_id: row.get("student_id"),
                ell: row.get("ell"),
                grade: row.get("grade"),
                teacher: row.get("teacher"),
                iep: row.get("iep"),
                student_504: row.get("student_504"),
                readplan: row.get("readplan"),
                gt: row.get("gt"),
                intervention: row.get("intervention"),
                eye_glasses: row.get("eye_glasses"),
            };

            Ok(student)

        }

        pub async fn update_student(firstname: String, lastname: String, gender: GenderEnum, date_of_birth: NaiveDate, student_id: i32, ell: ELLEnum, grade: GradeEnum, teacher: String, iep: bool, student_504: bool, readplan: bool, gt: bool, intervention: bool, eye_glasses: bool, pool: &PgPool) -> Result<Option<Student>, ServerFnError> {

            let row = sqlx::query("UPDATE students SET firstname =$1, lastname =$2, gender =$3::gender_enum, date_of_birth=$4::DATE, student_id=$5, ell =$6::ell_enum, grade =$7::grade_enum, teacher =$8, iep =$9, student_504 =$10, readplan =$11, gt =$12, intervention =$13, eye_glasses =$14 WHERE student_id = $5 RETURNING firstname, lastname, gender, date_of_birth, student_id, ell, grade, teacher, iep, student_504, readplan, gt, intervention, eye_glasses")
                .bind(firstname)
                .bind(lastname)
                .bind(gender)
                .bind(date_of_birth)
                .bind(student_id)
                .bind(ell)
                .bind(grade)
                .bind(teacher)
                .bind(iep)
                .bind(student_504)
                .bind(readplan)
                .bind(gt)
                .bind(intervention)
                .bind(eye_glasses)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let student = Student {
                firstname: row.get("firstname"),
                lastname: row.get("lastname"),
                gender: row.get("gender"),
                date_of_birth: row.get::<chrono::NaiveDate,_>("date_of_birth"),
                student_id: row.get("student_id"),
                ell: row.get("ell"),
                grade: row.get("grade"),
                teacher: row.get("teacher"),
                iep: row.get("iep"),
                student_504: row.get("student_504"),
                readplan: row.get("readplan"),
                gt: row.get("gt"),
                intervention: row.get("intervention"),
                eye_glasses: row.get("eye_glasses"),
            };

            Ok(Some(student))
        }
    }
}
