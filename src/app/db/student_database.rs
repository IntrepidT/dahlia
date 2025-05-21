use leptos::ServerFnError;

use crate::app::models::student;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")]{

        use crate::app::models::Student;
        use crate::app::models::student::{GradeEnum, ESLEnum, GenderEnum, InterventionEnum};
        use log::{debug, error, info, warn};
        use chrono::NaiveDate;
        use leptos::*;
        use sqlx::PgPool;
        use sqlx::prelude::*;

        pub async fn get_all_students(pool: &PgPool) -> Result<Vec<Student>, ServerFnError>{
            let rows  = sqlx::query("SELECT firstname, lastname, preferred, gender, date_of_birth, student_id, esl, grade, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin FROM students ORDER BY lastname")
                .fetch_all(pool)
                .await?;

            let students: Vec<Student> = rows
                .into_iter()
                .map(|row| {
                    let firstname: String = row.get("firstname");
                    let lastname: String = row.get("lastname");
                    let preferred: String = row.get("preferred");
                    let gender: GenderEnum = row.get("gender");
                    let date_of_birth: chrono::NaiveDate = row.get::<NaiveDate, _>("date_of_birth");
                    let student_id: i32 = row.get("student_id");
                    let esl: ESLEnum = row.get("esl");
                    let grade: GradeEnum = row.get("grade");
                    let teacher: String = row.get("teacher");
                    let iep: bool = row.get("iep");
                    let bip: bool = row.get("bip");
                    let student_504: bool = row.get("student_504");
                    let readplan: bool = row.get("readplan");
                    let gt: bool = row.get("gt");
                    let intervention: Option<InterventionEnum> = row.get("intervention");
                    let eye_glasses: bool = row.get("eye_glasses");
                    let notes: String = row.get("notes");
                    let pin: i32 = row.get("pin");

                    Student {
                        firstname,
                        lastname,
                        preferred,
                        gender,
                        date_of_birth,
                        student_id,
                        esl,
                        grade,
                        teacher,
                        iep,
                        bip,
                        student_504,
                        readplan,
                        gt,
                        intervention,
                        eye_glasses,
                        notes,
                        pin,
                    }
                })
                .collect();
            Ok(students)
        }

        pub async fn get_certain_student(student_id: i32, pool: &PgPool) -> Result<Student, ServerFnError> {
            let row = sqlx::query("SELECT firstname, lastname, preferred, gender, date_of_birth, student_id, esl, grade, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin FROM students WHERE student_id = $1")
                .bind(&student_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let student = Student {
                firstname: row.get("firstname"),
                lastname: row.get("lastname"),
                preferred: row.get("preferred"),
                gender: row.get("gender"),
                date_of_birth: row.get("date_of_birth"),
                student_id: row.get("student_id"),
                esl: row.get("esl"),
                grade: row.get("grade"),
                teacher: row.get("teacher"),
                iep: row.get("iep"),
                bip: row.get("bip"),
                student_504: row.get("student_504"),
                readplan: row.get("readplan"),
                gt: row.get("gt"),
                intervention: row.get("intervention"),
                eye_glasses: row.get("eye_glasses"),
                notes: row.get("notes"),
                pin: row.get("pin"),
            };

            Ok(student)
        }

        pub async fn add_student(new_student: &Student, pool: &PgPool) -> Result<Student, ServerFnError>{
            let row = sqlx::query("INSERT INTO students (firstname, lastname, preferred, gender, date_of_birth, student_id, esl, grade, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin) VALUES($1, $2, $3, $4::gender_enum, $5, $6, $7::esl_enum, $8::grade_enum, $9, $10, $11, $12, $13, $14, $15::intervention_enum, $16, $17, $18) RETURNING firstname, lastname, preferred, gender, date_of_birth, student_id, esl, grade, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin")
                .bind(&new_student.firstname)
                .bind(&new_student.lastname)
                .bind(&new_student.preferred)
                .bind(&new_student.gender.to_string())
                .bind(&new_student.date_of_birth)
                .bind(&new_student.student_id)
                .bind(&new_student.esl.to_string())
                .bind(&new_student.grade.to_string())
                .bind(&new_student.teacher)
                .bind(&new_student.iep)
                .bind(&new_student.bip)
                .bind(&new_student.student_504)
                .bind(&new_student.readplan)
                .bind(&new_student.gt)
                .bind(&new_student.intervention)
                .bind(&new_student.eye_glasses)
                .bind(&new_student.notes)
                .bind(&new_student.pin)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let student = Student {
                firstname: row.get("firstname"),
                lastname: row.get("lastname"),
                preferred: row.get("preferred"),
                gender: row.get("gender"),
                date_of_birth: row.get("date_of_birth"),
                student_id: row.get("student_id"),
                esl: row.get("esl"),
                grade: row.get("grade"),
                teacher: row.get("teacher"),
                iep: row.get("iep"),
                bip: row.get("bip"),
                student_504: row.get("student_504"),
                readplan: row.get("readplan"),
                gt: row.get("gt"),
                intervention: row.get("intervention"),
                eye_glasses: row.get("eye_glasses"),
                notes: row.get("notes"),
                pin: row.get("pin"),
            };

            Ok(student)
        }

        pub async fn delete_student(firstname: String, lastname: String, student_id: i32, pool: &PgPool) -> Result<Student, ServerFnError> {
            let row = sqlx::query("DELETE FROM students WHERE firstname = $1 AND lastname = $2 AND student_id =$3 RETURNING firstname, lastname, preferred, gender, date_of_birth, student_id, esl, grade, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin")
                .bind(firstname)
                .bind(lastname)
                .bind(student_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let student = Student {
                firstname: row.get("firstname"),
                lastname: row.get("lastname"),
                preferred: row.get("preferred"),
                gender: row.get("gender"),
                date_of_birth: row.get::<chrono::NaiveDate,_>("date_of_birth"),
                student_id: row.get("student_id"),
                esl: row.get("esl"),
                grade: row.get("grade"),
                teacher: row.get("teacher"),
                iep: row.get("iep"),
                bip: row.get("bip"),
                student_504: row.get("student_504"),
                readplan: row.get("readplan"),
                gt: row.get("gt"),
                intervention: row.get("intervention"),
                eye_glasses: row.get("eye_glasses"),
                notes: row.get("notes"),
                pin: row.get("pin"),
            };

            Ok(student)

        }

        pub async fn update_student(firstname: String, lastname: String, preferred: String, gender: GenderEnum, date_of_birth: NaiveDate, student_id: i32, esl: ESLEnum, grade: GradeEnum, teacher: String, iep: bool, bip: bool, student_504: bool, readplan: bool, gt: bool, intervention: Option<InterventionEnum>, eye_glasses: bool, notes: String, pin: i32, pool: &PgPool) -> Result<Option<Student>, ServerFnError> {

            let row = sqlx::query("UPDATE students SET firstname =$1, lastname =$2, preferred =$3, gender =$4::gender_enum, date_of_birth=$5::DATE, student_id=$6, esl =$7::esl_enum, grade =$8::grade_enum, teacher =$9, iep =$10, bip =$11, student_504 =$12, readplan =$13, gt =$14, intervention =$15::intervention_enum, eye_glasses =$16, notes =$17, pin =$18 WHERE student_id = $6 RETURNING firstname, lastname, preferred, gender, date_of_birth, student_id, esl, grade, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin")
                .bind(firstname)
                .bind(lastname)
                .bind(preferred)
                .bind(gender)
                .bind(date_of_birth)
                .bind(student_id)
                .bind(esl)
                .bind(grade)
                .bind(teacher)
                .bind(iep)
                .bind(bip)
                .bind(student_504)
                .bind(readplan)
                .bind(gt)
                .bind(intervention)
                .bind(eye_glasses)
                .bind(notes)
                .bind(pin)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let student = Student {
                firstname: row.get("firstname"),
                lastname: row.get("lastname"),
                preferred: row.get("preferred"),
                gender: row.get("gender"),
                date_of_birth: row.get::<chrono::NaiveDate,_>("date_of_birth"),
                student_id: row.get("student_id"),
                esl: row.get("esl"),
                grade: row.get("grade"),
                teacher: row.get("teacher"),
                iep: row.get("iep"),
                bip: row.get("bip"),
                student_504: row.get("student_504"),
                readplan: row.get("readplan"),
                gt: row.get("gt"),
                intervention: row.get("intervention"),
                eye_glasses: row.get("eye_glasses"),
                notes: row.get("notes"),
                pin: row.get("pin"),
            };

            Ok(Some(student))
        }

        pub async fn bulk_insert_students(students: Vec<Student>, pool: &PgPool) -> Result<Vec<Student>, ServerFnError> {
            // Start a database transaction for bulk insert
            let mut tx = pool.begin().await?;

            let mut inserted_students = Vec::new();

            // Prepare the bulk insert query
            for student in students {
                // Use the existing add_student logic within the transaction
                let row = sqlx::query("INSERT INTO students (firstname, lastname, preferred, gender, date_of_birth, student_id, esl, grade, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin) VALUES($1, $2, $3, $4::gender_enum, $5, $6, $7::esl_enum, $8::grade_enum, $9, $10, $11, $12, $13, $14, $15::intervention_enum, $16, $17, $18) RETURNING firstname, lastname, preferred, gender, date_of_birth, student_id, esl, grade, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin")
                    .bind(&student.firstname)
                    .bind(&student.lastname)
                    .bind(&student.preferred)
                    .bind(&student.gender.to_string())
                    .bind(&student.date_of_birth)
                    .bind(&student.student_id)
                    .bind(&student.esl.to_string())
                    .bind(&student.grade.to_string())
                    .bind(&student.teacher)
                    .bind(&student.iep)
                    .bind(&student.bip)
                    .bind(&student.student_504)
                    .bind(&student.readplan)
                    .bind(&student.gt)
                    .bind(&student.intervention)
                    .bind(&student.eye_glasses)
                    .bind(&student.notes)
                    .bind(&student.pin)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| ServerFnError::new(format!("Bulk insert error: {}", e)))?;

                let inserted_student = Student {
                    firstname: row.get("firstname"),
                    lastname: row.get("lastname"),
                    preferred: row.get("preferred"),
                    gender: row.get("gender"),
                    date_of_birth: row.get("date_of_birth"),
                    student_id: row.get("student_id"),
                    esl: row.get("esl"),
                    grade: row.get("grade"),
                    teacher: row.get("teacher"),
                    iep: row.get("iep"),
                    bip: row.get("bip"),
                    student_504: row.get("student_504"),
                    readplan: row.get("readplan"),
                    gt: row.get("gt"),
                    intervention: row.get("intervention"),
                    eye_glasses: row.get("eye_glasses"),
                    notes: row.get("notes"),
                    pin: row.get("pin"),
                };

                inserted_students.push(inserted_student);
            }

            // Commit the transaction
            tx.commit().await?;

            Ok(inserted_students)
        }
    }
}
