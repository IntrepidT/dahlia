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

            let rows = sqlx::query("SELECT word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker, weighted_multiple_choice FROM question_table WHERE testlinker = $1::uuid ORDER BY qnumber ASC")
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
                    let weighted_multiple_choice: Option<String> = row.get("weighted_multiple_choice");

                    let testlinker = testlinker_one.to_string();

                    Question {
                        word_problem,
                        point_value,
                        question_type,
                        options,
                        correct_answer,
                        qnumber,
                        testlinker,
                        weighted_options: weighted_multiple_choice, // Map from database field
                    }
                })
                .collect();
            Ok(questions)
        }

        pub async fn add_question(question: &Question, pool: &sqlx::PgPool)-> Result<Question, ServerFnError> {
            let testlinker_uuid = Uuid::parse_str(&question.testlinker).map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let row = sqlx::query("INSERT INTO question_table (word_problem, point_value, question_type, options, correct_answer, testlinker, weighted_multiple_choice) VALUES($1, $2, $3::questiontype_enum, $4, $5, $6::uuid, $7) RETURNING word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker::text, weighted_multiple_choice")
                .bind(&question.word_problem)
                .bind(&question.point_value)
                .bind(&question.question_type)
                .bind(&question.options)
                .bind(&question.correct_answer)
                .bind(testlinker_uuid)
                .bind(&question.weighted_options) // Include weighted_options in INSERT
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
                weighted_options: row.get("weighted_multiple_choice"), // Map from database field
            };

            Ok(question)
        }


        pub async fn update_question(question: &Question, pool: &sqlx::PgPool) -> Result<Option<Question>, ServerFnError> {
            let testlinker_uuid = Uuid::parse_str(&question.testlinker).map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let row = sqlx::query("UPDATE question_table SET word_problem = $1, point_value = $2, question_type = $3::questiontype_enum, options = $4, correct_answer = $5, weighted_multiple_choice = $6 WHERE qnumber = $7 AND testlinker = $8::uuid RETURNING word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker::text, weighted_multiple_choice")
                .bind(&question.word_problem)
                .bind(&question.point_value)
                .bind(&question.question_type)
                .bind(&question.options)
                .bind(&question.correct_answer)
                .bind(&question.weighted_options) // Include weighted_options in UPDATE
                .bind(&question.qnumber)
                .bind(testlinker_uuid)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let question: Question = Question {
                word_problem: row.get("word_problem"),
                point_value: row.get("point_value"),
                question_type: row.get("question_type"),
                options: row.get("options"),
                correct_answer: row.get("correct_answer"),
                qnumber: row.get("qnumber"),
                testlinker: row.get("testlinker"),
                weighted_options: row.get("weighted_multiple_choice"), // Map from database field
            };
            Ok(Some(question))
        }


        pub async fn delete_all_questions(test_id: String, pool: &PgPool) -> Result<Vec<Question>, ServerFnError> {
            let testlinker = Uuid::parse_str(&test_id).expect("This did not convert to a UUID correctly");

            let rows = sqlx::query("DELETE FROM question_table WHERE testlinker = $1 RETURNING word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker, weighted_multiple_choice")
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
                    let weighted_multiple_choice: Option<String> = row.get("weighted_multiple_choice");

                    let testlinker = testlinker_one.to_string();

                    Question {
                        word_problem,
                        point_value,
                        question_type,
                        options,
                        correct_answer,
                        qnumber,
                        testlinker,
                        weighted_options: weighted_multiple_choice, // Map from database field
                    }
                })
                .collect();
            Ok(questions)
        }
        pub async fn delete_question(qnumber: i32, test_id: String, pool: &PgPool) -> Result<Question, ServerFnError> {
            let testlinker = Uuid::parse_str(&test_id).expect("This did not convert to a UUID correctly");

            let row = sqlx::query("DELETE FROM question_table WHERE qnumber = $1 AND testlinker = $2 RETURNING word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker, weighted_multiple_choice")
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
                weighted_options: row.get("weighted_multiple_choice"), // Map from database field
            };

            Ok(deleted_question)
        }

        pub async fn get_single_question(qnumber: i32, test_id: String, pool: &PgPool) -> Result<Question, ServerFnError> {
            let testlinker_uuid = Uuid::parse_str(&test_id).map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let row = sqlx::query("SELECT word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker, weighted_multiple_choice FROM question_table WHERE qnumber = $1 AND testlinker = $2")
                .bind(qnumber)
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
                weighted_options: row.get("weighted_multiple_choice"), // Map from database field
            };

            Ok(question)
        }

        pub async fn update_question_options(qnumber: i32, test_id: String, new_options: Vec<String>, pool: &PgPool) -> Result<Question, ServerFnError> {
            let testlinker_uuid = Uuid::parse_str(&test_id).map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let row = sqlx::query("UPDATE question_table SET options = $1 WHERE qnumber = $2 AND testlinker = $3 RETURNING word_problem, point_value, question_type, options, correct_answer, qnumber, testlinker, weighted_multiple_choice")
                .bind(&new_options)
                .bind(qnumber)
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
                weighted_options: row.get("weighted_multiple_choice"), // Map from database field
            };

            Ok(question)
        }

        pub async fn count_questions_by_test(test_id: String, pool: &PgPool) -> Result<i64, ServerFnError> {
            let testlinker_uuid = Uuid::parse_str(&test_id).map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM question_table WHERE testlinker = $1")
                .bind(testlinker_uuid)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            Ok(count)
        }

        pub async fn count_multiple_choice_questions(test_id: String, pool: &PgPool) -> Result<i64, ServerFnError> {
            let testlinker_uuid = Uuid::parse_str(&test_id).map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM question_table WHERE testlinker = $1 AND question_type = 'MultipleChoice'")
                .bind(testlinker_uuid)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            Ok(count)
        }
    }
}
