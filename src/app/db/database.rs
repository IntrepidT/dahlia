use crate::app::models::student::{ELLEnum, GenderEnum, GradeEnum};

cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")] {

        use crate::app::models::{Student, Test, TestType, QuestionType, Question};
        use crate::app::errors::{ErrorMessage, StudentError, TestError, ErrorMessageTest, QuestionError, ErrorMessageQuestion};
        use uuid::{Uuid, uuid};
        use log::{debug, error, info, warn};
        use sqlx::prelude::*;
        use chrono::NaiveDate;
        use sqlx::{Encode, Decode, Postgres, Type};
        use sqlx::encode::{IsNull};
        use sqlx::postgres::{PgArgumentBuffer};
        use sqlx::{PgPool};
        use std::env;
        use leptos::ServerFnError;
        use dotenvy::dotenv;
        use sqlx::postgres::{PgPoolOptions, PgValueRef};
        use std::error::Error;
        use std::str::FromStr;
        use tokio::*;

       pub async fn create_pool() -> sqlx::PgPool {
           dotenv().ok();
            let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

            //create the connection pool using sqlx
            let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url.as_str())
            .await
            .expect("Failed to create PostgreSQL pool");

            sqlx::migrate!()
                .run(&pool)
                .await
                .expect("migrations failed");

           pool
        }

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

        pub async fn smart_search_all_students(fragment: String, pool: &PgPool)-> Result<Vec<Student>, ServerFnError> {
            let rows = sqlx::query("SELECT firstname, lastname, gender, date_of_birth, student_id, ell, grade, teacher, iep, student_504, readplan, gt, intervention, eye_glasses FROM students WHERE firstname % $1 OR lastname % $1 OR teacher % $1")
                .bind(fragment)
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

        pub async fn get_all_tests(pool: &sqlx::PgPool) -> Result<Vec<Test>, ServerFnError>{
           let rows = sqlx::query("SELECT name, score, comments, testarea, test_id::text FROM tests ORDER BY name DESC")
               .fetch_all(pool)
               .await?;

            let tests: Vec<Test> = rows
                .into_iter()
                .map(|row| {
                    let name: String = row.get("name");
                    let score: i32 = row.get("score");
                    let comments: String = row.get("comments");
                    let testarea: TestType = row.get("testarea");
                    let test_id  = row.get::<String,_>("test_id");

                    Test {
                        name,
                        score,
                        comments,
                        testarea,
                        test_id
                    }
                })
                .collect();
            Ok(tests)
        }

        pub async fn get_all_questions(test_id: String, pool: &sqlx::PgPool) -> Result<Vec<Question>, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).expect("Invalid UUID format");

            let rows = sqlx::query("SELECT word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker FROM question_table WHERE testlinker = $1::uuid")
                .bind(&ID)
                .fetch_all(pool)
                .await?;

            let questions: Vec<Question> = rows
                .into_iter()
                .map(|row| {
                    let word_problem: String = row.get("word_problem");
                    let point_value: i32 = row.get("point_value");
                    let question_type: QuestionType = row.get("question_type");
                    let options: Vec<String> = row.get("options");
                    let correct_answer: String = row.get("correct_answer");
                    let qnumber: i32 = row.get("qnumber");
                    let testlinker_one: Uuid = row.get("testlinker");

                    let testlinker = testlinker_one.to_string();

                    Question {
                        word_problem,
                        point_value,
                        question_type,
                        options,
                        correct_answer,
                        qnumber,
                        testlinker,
                    }
                })
                .collect();
            Ok(questions)
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

        pub async fn add_test(new_test: &Test, pool: &PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&new_test.test_id)
                .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let row = sqlx::query("INSERT INTO tests (name, score, comments, testarea, test_id) VALUES($1, $2, $3, $4::testarea_enum, $5::uuid) RETURNING name, score, comments, testarea, test_id::text") .bind(&new_test.name).bind(&new_test.score).bind(&new_test.comments).bind(&new_test.testarea).bind(&ID)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let test = Test {
                    name: row.get("name"),
                    score: row.get("score"),
                    comments: row.get("comments"),
                    testarea: row.get("testarea"),
                    test_id: row.get("test_id"),
            };

            Ok(test)
        }

        pub async fn add_question(question: &Question, pool: &sqlx::PgPool)-> Result<Question, ServerFnError> {
            let testlinker_uuid = Uuid::parse_str(&question.testlinker).map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let row = sqlx::query("INSERT INTO question_table (word_problem, point_value, question_type, options, correct_answer, testlinker) VALUES($1, $2, $3::questiontype_enum, $4, $5, $6::uuid) RETURNING word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker::text")
                .bind(&question.word_problem)
                .bind(&question.point_value)
                .bind(&question.question_type)
                .bind(&question.options)
                .bind(&question.correct_answer)
                .bind(testlinker_uuid)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let question = Question {
                word_problem: row.get("word_problem"),
                point_value: row.get("point_value"),
                question_type: row.get("question_type"),
                options: row.get("options"),
                correct_answer: row.get("correct_answer"),
                qnumber: row.get("qnumber"),
                testlinker: row.get("testlinker"),
            };

            Ok(question)
        }

        pub async fn update_test(test: &Test, pool: &sqlx::PgPool) -> Result<Option<Test>, ServerFnError> {
            let query = "UPDATE tests SET name =$1, score =$2, comments =$3, testarea =$4::testarea_enum, questions =$5 WHERE test_id =$6";
            let ID = Uuid::parse_str(&test.test_id).expect("The UUID conversion did not occur correctly");

            let row = sqlx::query("UPDATE tests SET name =$1, score =$2, comments =$3, testarea =$4::testarea_enum WHERE test_id =$5 RETURNING name, score, comments, testarea, test_id::text")
                .bind(&test.name)
                .bind(&test.score)
                .bind(&test.comments)
                .bind(&test.testarea.to_string())
                .bind(ID)
                .fetch_one(pool)
                .await?;

            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                test_id: row.get("test_id"),
            };

            Ok(Some(test))
        }

        pub async fn update_question(question: &Question, pool: &sqlx::PgPool) -> Result<Option<Question>, ServerFnError> {
            let query = "UPDATE question_table SET word_problem =$1, point_value = $2, question_type = $3::questiontype_enum, options = $4, correct_answer =$5 WHERE qnumber = $6 AND testlinker = $7";
            let row = sqlx::query("UPDATE question_table SET word_problem =$1, point_value = $2, question_type = $3::questiontype_enum, options = $4, correct_answer =$5 WHERE qnumber = $6 AND testlinker = $7")
                .bind(&question.word_problem)
                .bind(&question.point_value)
                .bind(&question.question_type.to_string())
                .bind(&question.options)
                .bind(&question.correct_answer)
                .bind(&question.qnumber)
                .bind(&question.testlinker)
                .fetch_one(pool)
                .await?;

            let question: Question = Question {
                word_problem: row.get("word_problem"),
                point_value: row.get("point_value"),
                question_type: row.get("question_type"),
                options: row.get("options"),
                correct_answer: row.get("correct_answer"),
                qnumber: row.get("qnumber"),
                testlinker: row.get("testlinker"),
            };
            Ok(Some(question))
        }

        pub async fn delete_test(test_id: String, pool: &sqlx::PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).expect("The test_id did not correctly become a UUID");
            let row = sqlx::query("DELETE FROM tests WHERE test_id = $1 RETURNING name, score, comments, testarea, test_id::text")
                .bind(ID)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;


            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                test_id: row.get("test_id"),
            };

            Ok(test)
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

        pub async fn delete_question(qnumber: i32, test_id: String, pool: &PgPool) -> Result<Question, ServerFnError> {
            let testlinker = Uuid::parse_str(&test_id).expect("This did not convert to a UUID correctly");

            let row = sqlx::query("DELETE FROM question_table WHERE qnumber = $1 AND testlinker =$2 RETURNING word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker")
                .bind(&qnumber)
                .bind(&testlinker)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let deleted_question = Question {
                word_problem: row.get("word_problem"),
                point_value: row.get("point_value"),
                question_type: row.get("question_type"),
                options: row.get("options"),
                correct_answer: row.get("correct_answer"),
                qnumber: row.get("qnumber"),
                testlinker: row.get("testlinker"),
            };

            Ok(deleted_question)
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
