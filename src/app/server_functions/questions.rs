use crate::app::errors::{question_errors, ErrorMessageQuestion, ResponseErrorTraitQuestion};
use crate::app::models::{
    question::{Question, QuestionType},
    CreateNewQuestionRequest, DeleteQuestionRequest, UpdateQuestionRequest,
};
use leptos::*;
use log::{debug, info, warn};
#[cfg(feature = "ssr")]
use {crate::app::db::database, actix_web::web, sqlx::PgPool, std::error::Error, uuid::Uuid};

#[server(GetQuestions, "/api")]
pub async fn get_questions(test_id: String) -> Result<Vec<Question>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;
        log::info!("Attempting to retrieve all tests from database");

        match database::get_all_questions(test_id, &pool).await {
            Ok(questions) => {
                log::info!("Successfully retrieved all tests from database");
                Ok(questions)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(AddQuestion, "/api")]
pub async fn add_question(
    test_id: String,
    add_question_request: CreateNewQuestionRequest,
) -> Result<Question, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to add new question to the database");

        let buffer_question = Question::new(
            add_question_request.word_problem,
            add_question_request.point_value,
            add_question_request.question_type,
            add_question_request.options,
            add_question_request.correct_answer,
            0, //this value is technically the qnumber but qnumber is determined by the backend
            test_id.clone(),
        );

        match database::add_question(&buffer_question, &pool).await {
            Ok(created_question) => {
                log::info!(
                    "Successfully created question with ID: {}",
                    created_question.testlinker
                );
                Ok(created_question)
            }
            Err(e) => {
                log::info!("Failed to create question: {:?}", e);
                Err(ServerFnError::new(format!(
                    "The question created was not a question"
                )))
            }
        }
    }
}

#[server(DeleteQuestion, "/api")]
pub async fn delete_question(
    delete_question_request: DeleteQuestionRequest,
) -> Result<Question, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to delete question from the database");

        match database::delete_question(
            delete_question_request.qnumber,
            delete_question_request.testlinker,
            &pool,
        )
        .await
        {
            Ok(deleted) => Ok(deleted),
            Err(_) => Err(ServerFnError::new(
                "Failed to delete question from the database",
            )),
        }
    }
}

#[server(EditQuestion, "/api")]
pub async fn edit_question(
    test_id: String,
    edit_question_request: UpdateQuestionRequest,
) -> Result<Question, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to update question from the database");

        let buffer_question = Question::new(
            edit_question_request.word_problem,
            edit_question_request.point_value,
            edit_question_request.question_type,
            edit_question_request.options,
            edit_question_request.correct_answer,
            edit_question_request.qnumber,
            edit_question_request.testlinker,
        );

        match database::update_question(&buffer_question, &pool).await {
            Ok(Some(updated_student)) => Ok(updated_student),
            Ok(None) => Err(ServerFnError::new(format!(
                "Failed to correctly existing student in the database"
            ))),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update student: {}",
                e
            ))),
        }
    }
}

/*cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {

        use crate::app::db::database;
        use crate::app::errors::QuestionError;
        use sqlx::PgPool;

        pub async fn retrieve_all_questions(test_id: String, pool: &sqlx::PgPool) -> Vec<Question> {

            let get_all_question_results = database::get_all_questions(test_id.clone(), pool).await;

            get_all_question_results.expect("There was a problem gathering all the questions for this test.")
        }

        pub async fn add_new_question<T> (word_problem: T, point_value: i32, question_type: QuestionType, options: Vec<String>, correct_answer: T, qnumber: i64, test_id: T, pool: &sqlx::PgPool) -> Result<Question, ServerFnErro> where T: Into<String> {
            let new_question = Question::new(
                word_problem.into(),
                point_value,
                question_type,
                options,
                correct_answer.into(),
                qnumber,
                test_id.into(),
            );

            database::add_question(&new_question, pool).await
        }
        pub async fn delete_certain_question(qnumber: i64, test_id: String, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
            database::delete_question(qnumber, test_id, pool).await
        }

        pub async fn edit_certain_question<T>(word_problem: T, point_value: i32, question_type: QuestionType, options:Vec<String>, correct_answer: T, qnumber: i64, test_id: T, pool: &sqlx::PgPool) -> Result<Option<Question>, sqlx::Error> where T: Into<String> {
            let updated_question = Question::new(
                word_problem.into(),
                point_value,
                question_type,
                options,
                correct_answer.into(),
                qnumber,
                test_id.into(),
            );
            database::update_question(&updated_question, pool).await
        }
    }
}*/
