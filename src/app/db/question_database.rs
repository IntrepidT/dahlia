cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")] {

        use crate::app::models::{Student, Test, TestType, QuestionType, Question, Score, CreateScoreRequest};
        use crate::app::errors::{ErrorMessage, StudentError, TestError, ErrorMessageTest, QuestionError, ErrorMessageQuestion};
        use leptos::*;
        use sqlx::PgPool;
        use uuid::{Uuid, uuid};
        use log::{debug, error, info, warn};
        use sqlx::prelude::*;

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




        pub async fn delete_all_questions(test_id: String, pool: &PgPool) -> Result<Vec<Question>, ServerFnError> {
            let testlinker = Uuid::parse_str(&test_id).expect("This did not convert to a UUID correctly");

            let rows = sqlx::query("DELETE FROM question_table WHERE testlinker = $1 RETURNING word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker")
                .bind(&testlinker)
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

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

    }
}
