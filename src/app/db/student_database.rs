use crate::app::models::student;
use leptos::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")]{

        use crate::app::models::{Student, AddStudentRequest};
        use crate::app::models::student::{GradeEnum, ESLEnum, GenderEnum, InterventionEnum};
        use log::{debug, error, info, warn};
        use chrono::NaiveDate;
        use sqlx::PgPool;
        use sqlx::prelude::*;

        pub async fn get_all_students(pool: &PgPool) -> Result<Vec<Student>, ServerFnError>{
            let rows  = sqlx::query("SELECT firstname, lastname, preferred, gender, date_of_birth, student_id, esl, current_grade_level, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin FROM students ORDER BY lastname")
                .fetch_all(pool)
                .await?;

            let students: Vec<Student> = rows
                .into_iter()
                .map(|row| {
                    let firstname: Option<String> = row.get("firstname");
                    let lastname: Option<String> = row.get("lastname");
                    let preferred: String = row.get("preferred");
                    let gender: GenderEnum = row.get("gender");
                    let date_of_birth: chrono::NaiveDate = row.get::<NaiveDate, _>("date_of_birth");
                    let student_id: i32 = row.get("student_id");
                    let esl: ESLEnum = row.get("esl");
                    let current_grade_level: GradeEnum = row.get("current_grade_level");
                    let teacher: String = row.get("teacher");
                    let iep: bool = row.get("iep");
                    let bip: bool = row.get("bip");
                    let student_504: bool = row.get("student_504");
                    let readplan: bool = row.get("readplan");
                    let gt: bool = row.get("gt");
                    let intervention: Option<InterventionEnum> = row.get("intervention");
                    let eye_glasses: bool = row.get("eye_glasses");
                    let notes: String = row.get("notes");
                    let pin: Option<i32> = row.get("pin");

                    Student {
                        firstname,
                        lastname,
                        preferred,
                        gender,
                        date_of_birth,
                        student_id,
                        esl,
                        current_grade_level,
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
            let row = sqlx::query("SELECT firstname, lastname, preferred, gender, date_of_birth, student_id, esl, current_grade_level, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin FROM students WHERE student_id = $1")
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
                current_grade_level: row.get("current_grade_level"),
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
            let row = sqlx::query("INSERT INTO students (firstname, lastname, preferred, gender, date_of_birth, student_id, esl, current_grade_level, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin) VALUES($1, $2, $3, $4::gender_enum, $5, $6, $7::esl_enum, $8::grade_enum, $9, $10, $11, $12, $13, $14, $15::intervention_enum, $16, $17, $18) RETURNING firstname, lastname, preferred, gender, date_of_birth, student_id, esl, current_grade_level, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin")
                .bind(&new_student.firstname)
                .bind(&new_student.lastname)
                .bind(&new_student.preferred)
                .bind(&new_student.gender.to_string())
                .bind(&new_student.date_of_birth)
                .bind(&new_student.student_id)
                .bind(&new_student.esl.to_string())
                .bind(&new_student.current_grade_level.to_string())
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
                current_grade_level: row.get("current_grade_level"),
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
            let row = sqlx::query("DELETE FROM students WHERE firstname = $1 AND lastname = $2 AND student_id =$3 RETURNING firstname, lastname, preferred, gender, date_of_birth, student_id, esl, current_grade_level, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin")
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
                current_grade_level: row.get("current_grade_level"),
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

        pub async fn update_student(firstname: String, lastname: String, preferred: String, gender: GenderEnum, date_of_birth: NaiveDate, student_id: i32, esl: ESLEnum, current_grade_level: GradeEnum, teacher: String, iep: bool, bip: bool, student_504: bool, readplan: bool, gt: bool, intervention: Option<InterventionEnum>, eye_glasses: bool, notes: String, pin: i32, pool: &PgPool) -> Result<Option<Student>, ServerFnError> {

            let row = sqlx::query("UPDATE students SET firstname =$1, lastname =$2, preferred =$3, gender =$4::gender_enum, date_of_birth=$5::DATE, student_id:$6, esl =$7::esl_enum, current_grade_level =$8::grade_enum, teacher =$9, iep =$10, bip =$11, student_504 =$12, readplan =$13, gt =$14, intervention =$15::intervention_enum, eye_glasses =$16, notes =$17, pin =$18 WHERE student_id = $6 RETURNING firstname, lastname, preferred, gender, date_of_birth, student_id, esl, current_grade_level, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin")
                .bind(firstname)
                .bind(lastname)
                .bind(preferred)
                .bind(gender)
                .bind(date_of_birth)
                .bind(student_id)
                .bind(esl)
                .bind(current_grade_level)
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
                current_grade_level: row.get("current_grade_level"),
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

        pub async fn bulk_insert_students(students: Vec<AddStudentRequest>, pool: &PgPool) -> Result<Vec<Student>, ServerFnError> {
            // Start a database transaction for bulk insert
            let mut tx = pool.begin().await?;

            let mut inserted_students = Vec::new();

            // Prepare the bulk insert query
            for student in students {
                // Use the existing add_student logic within the transaction
                let row = sqlx::query("INSERT INTO students (firstname, lastname, preferred, gender, date_of_birth, student_id, esl, current_grade_level, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin) VALUES($1, $2, $3, $4::gender_enum, $5, $6, $7::esl_enum, $8::grade_enum, $9, $10, $11, $12, $13, $14, $15::intervention_enum, $16, $17, $18) RETURNING firstname, lastname, preferred, gender, date_of_birth, student_id, esl, current_grade_level, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin")
                    .bind(&student.firstname)
                    .bind(&student.lastname)
                    .bind(&student.preferred)
                    .bind(&student.gender.to_string())
                    .bind(&student.date_of_birth)
                    .bind(&student.student_id)
                    .bind(&student.esl.to_string())
                    .bind(&student.current_grade_level.to_string())
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
                    current_grade_level: row.get("current_grade_level"),
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

        pub async fn bulk_insert_students_optimized(students: Vec<AddStudentRequest>, pool: &PgPool) -> Result<usize, ServerFnError> {
            if students.is_empty() {
                return Ok(0);
            }

            // Start a database transaction
            let mut tx = pool.begin().await?;

            // Method 1: Using UNNEST for maximum efficiency (PostgreSQL specific)
            let result = bulk_insert_with_unnest(&students, &mut tx).await;

            match result {
                Ok(count) => {
                    tx.commit().await?;
                    Ok(count)
                }
                Err(e) => {
                    tx.rollback().await?;
                    Err(ServerFnError::new(format!("Bulk insert failed: {}", e)))
                }
            }
        }

        // Alternative method using batch inserts if UNNEST doesn't work
        pub async fn bulk_insert_students_batch(students: Vec<AddStudentRequest>, pool: &PgPool) -> Result<usize, ServerFnError> {
            if students.is_empty() {
                return Ok(0);
            }

            let mut tx = pool.begin().await?;
            let batch_size = 100; // Adjust based on your needs
            let mut total_inserted = 0;

            for chunk in students.chunks(batch_size) {
                let count = insert_batch(chunk, &mut tx).await?;
                total_inserted += count;
            }

            tx.commit().await?;
            Ok(total_inserted)
        }

        async fn bulk_insert_with_unnest(students: &[AddStudentRequest], tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<usize, sqlx::Error> {
            let mut firstnames = Vec::new();
            let mut lastnames = Vec::new();
            let mut preferreds = Vec::new();
            let mut genders = Vec::new();
            let mut dates_of_birth = Vec::new();
            let mut student_ids = Vec::new();
            let mut esls = Vec::new();
            let mut grades = Vec::new();
            let mut teachers = Vec::new();
            let mut ieps = Vec::new();
            let mut bips = Vec::new();
            let mut student_504s = Vec::new();
            let mut readplans = Vec::new();
            let mut gts = Vec::new();
            let mut interventions = Vec::new();
            let mut eye_glasses = Vec::new();
            let mut notes = Vec::new();
            let mut pins = Vec::new();

            for student in students {
                firstnames.push(&student.firstname);
                lastnames.push(&student.lastname);
                preferreds.push(&student.preferred);
                genders.push(student.gender.to_string());
                dates_of_birth.push(student.date_of_birth);
                student_ids.push(student.student_id);
                esls.push(student.esl.to_string());
                grades.push(student.current_grade_level.to_string());
                teachers.push(&student.teacher);
                ieps.push(student.iep);
                bips.push(student.bip);
                student_504s.push(student.student_504);
                readplans.push(student.readplan);
                gts.push(student.gt);
                interventions.push(student.intervention.as_ref().map(|i| i.to_string()));
                eye_glasses.push(student.eye_glasses);
                notes.push(&student.notes);
                pins.push(student.pin);
            }

            let query = r#"
                INSERT INTO students (
                    firstname, lastname, preferred, gender, date_of_birth, student_id, 
                    esl, current_grade_level, teacher, iep, bip, student_504, readplan, gt, 
                    intervention, eye_glasses, notes, pin
                )
                SELECT * FROM UNNEST(
                    $1::text[], $2::text[], $3::text[], $4::gender_enum[], $5::date[], $6::int4[],
                    $7::esl_enum[], $8::grade_enum[], $9::text[], $10::bool[], $11::bool[], $12::bool[],
                    $13::bool[], $14::bool[], $15::intervention_enum[], $16::bool[], $17::text[], $18::int4[]
                )
                ON CONFLICT (student_id) DO UPDATE SET
                    firstname = EXCLUDED.firstname,
                    lastname = EXCLUDED.lastname,
                    preferred = EXCLUDED.preferred,
                    gender = EXCLUDED.gender,
                    date_of_birth = EXCLUDED.date_of_birth,
                    esl = EXCLUDED.esl,
                    current_grade_level = EXCLUDED.current_grade_level,
                    teacher = EXCLUDED.teacher,
                    iep = EXCLUDED.iep,
                    bip = EXCLUDED.bip,
                    student_504 = EXCLUDED.student_504,
                    readplan = EXCLUDED.readplan,
                    gt = EXCLUDED.gt,
                    intervention = EXCLUDED.intervention,
                    eye_glasses = EXCLUDED.eye_glasses,
                    notes = EXCLUDED.notes,
                    pin = EXCLUDED.pin
            "#;

            let result = sqlx::query(query)
                .bind(&firstnames)
                .bind(&lastnames)
                .bind(&preferreds)
                .bind(&genders)
                .bind(&dates_of_birth)
                .bind(&student_ids)
                .bind(&esls)
                .bind(&grades)
                .bind(&teachers)
                .bind(&ieps)
                .bind(&bips)
                .bind(&student_504s)
                .bind(&readplans)
                .bind(&gts)
                .bind(&interventions)
                .bind(&eye_glasses)
                .bind(&notes)
                .bind(&pins)
                .execute(&mut **tx)
                .await?;

            Ok(result.rows_affected() as usize)
        }

        async fn insert_batch(students: &[AddStudentRequest], tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<usize, sqlx::Error> {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO students (firstname, lastname, preferred, gender, date_of_birth, student_id, esl, current_grade_level, teacher, iep, bip, student_504, readplan, gt, intervention, eye_glasses, notes, pin) "
            );

            query_builder.push_values(students, |mut b, student| {
                b.push_bind(&student.firstname)
                 .push_bind(&student.lastname)
                 .push_bind(&student.preferred)
                 .push_bind(student.gender.to_string())
                 .push_bind(student.date_of_birth)
                 .push_bind(student.student_id)
                 .push_bind(student.esl.to_string())
                 .push_bind(student.current_grade_level.to_string())
                 .push_bind(&student.teacher)
                 .push_bind(student.iep)
                 .push_bind(student.bip)
                 .push_bind(student.student_504)
                 .push_bind(student.readplan)
                 .push_bind(student.gt)
                 .push_bind(&student.intervention)
                 .push_bind(student.eye_glasses)
                 .push_bind(&student.notes)
                 .push_bind(student.pin);
            });

            // Add conflict resolution if needed
            query_builder.push(" ON CONFLICT (student_id) DO UPDATE SET firstname = EXCLUDED.firstname, lastname = EXCLUDED.lastname");

            let query = query_builder.build();
            let result = query.execute(&mut **tx).await?;

            Ok(result.rows_affected() as usize)
        }
    }
}
