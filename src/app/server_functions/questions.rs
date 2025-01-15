use crate::app::errors::{ErrorMessageQuestion, ResponseErrorTraitQuestion};
use crate::app::models::{
    question::{Question, QuestionType},
    CreateNewQuestionRequest, DeleteQuestionRequest, EditQuestionRequest,
};
use leptos::*;
use serde::*;

#[server(GetQuestions, "/api")]
pub async fn get_questions(test_identifier: i64) -> Result<Vec<Question>, ServerFnError> {
    let questions = retrieve_all_questions(test_identifier.clone()).await;
    Ok(questions)
}

#[server(AddQuestion, "/api")]
pub async fn add_question(
    question_id: i64,
    add_question_request: CreateNewQuestionRequest,
) -> Result<Question, ServerFnError> {
    let new_question = add_new_question(
        question_id,
        add_question_request.word_problem,
        add_question_request.point_value,
        add_question_request.qtype,
        add_question_request.options,
        add_question_request.correct_answer,
        add_question_request.comments,
        add_question_request.qnumber,
    )
    .await;

    match new_question {
        Some(created_question) => Ok(created_question),
        None => Err(ServerFnError::Args(String::from(
            "Error in creating the question!",
        ))),
    }
}

#[server(DeleteQuestion, "/api")]
pub async fn delete_question(
    delete_question_request: DeleteQuestionRequest,
) -> Result<Question, ServerFnError> {
    let deleted_results = delete_certain_question(delete_question_request.qnumber).await;

    match deleted_results {
        Ok(deleted) => {
            if let Some(deleted_test) = deleted {
                Ok(deleted_test)
            } else {
                Err(ServerFnError::Response(ErrorMessageQuestion::create(
                    QuestionError::QuestionDeleteFailure,
                )))
            }
        }
        Err(question_error) => Err(ServerFnError::Response(ErrorMessageQuestion::create(
            question_error,
        ))),
    }
}

#[server(EditQuestion, "/api")]
pub async fn edit_question(
    test_id: i64,
    edit_question_request: EditQuestionRequest,
) -> Result<Question, ServerFnError> {
    let updated = edit_certain_question(
        test_id,
        edit_question_request.word_problem,
        edit_question_request.point_value,
        edit_question_request.qtype,
        edit_question_request.options,
        edit_question_request.correct_answer,
        edit_question_request.comments,
        edit_question_request.qnumber,
    )
    .await;

    match updated {
        Ok(updated_result) => {
            if let Some(updated_question) = updated_result {
                Ok(updated_question)
            } else {
                Err(ServerFnError::Args(ErrorMessageQuestion::create(
                    QuestionError::QuestionUpdateFailure,
                )))
            }
        }
        Err(question_error) => Err(ServerFnError::Args(ErrorMessageQuestion::create(
            question_error,
        ))),
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {

        use crate::app::db::database;
        use crate::app::errors::QuestionError;

        pub async fn retrieve_all_questions(test_identifier: i64) -> Vec<Question> {

            let get_all_question_results = database::get_all_test_questions(test_identifier.clone()).await;
            match get_all_question_results {
                Some(found_question) => found_question,
                None => Vec::new()
            }
        }

        pub async fn add_new_question<T> (test_id: i64, word_problem: T, point_value: i32, qtype: QuestionType, options: Vec<String>, correct_answer: T, comments: T, qnumber: i64) -> Option<Question> where T: Into<String> {
            let new_question = Question::new(
                word_problem.into(),
                point_value,
                qtype,
                options,
                correct_answer.into(),
                comments.into(),
                qnumber,
            );

            database::add_question_test(test_id, new_question).await
        }
        pub async fn delete_certain_question(qnumber: i64) -> Result<Option<Question>, QuestionError> {
            database::delete_question(qnumber).await
        }

        pub async fn edit_certain_question<T>(test_id: i64, word_problem: T, point_value: i32, qtype: QuestionType, options:Vec<String>, correct_answer: T, comments: T, qnumber: i64) -> Result<Option<Question>, QuestionError> where T: Into<String> {
            database::update_question(test_id, word_problem.into(), point_value, qtype, options, correct_answer.into(), comments.into(), qnumber).await
        }
    }
}
