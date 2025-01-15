use crate::app::errors::{ErrorMessageTest, ResponseErrorTraitTest};
use crate::app::models::{
    test::{test_type, Test},
    CreateNewTestRequest, DeleteTestRequest, EditTestRequest,
};
use leptos::*;
use serde::*;

#[server(GetTests, "/api")]
pub async fn get_tests() -> Result<Vec<Test>, ServerFnError> {
    let tests = retrieve_all_tests().await;
    Ok(tests)
}

#[server(AddTest, "/api")]
pub async fn add_test(add_test_request: CreateNewTestRequest) -> Result<Test, ServerFnError> {
    let new_test = add_new_test(
        add_test_request.name,
        add_test_request.score,
        add_test_request.comments,
        add_test_request.test_area,
        add_test_request.test_identifier,
    )
    .await;

    match new_test {
        Some(created_test) => Ok(created_test),
        None => Err(ServerFnError::Args(String::from(
            "Error in creating the test!",
        ))),
    }
}

#[server(DeleteTest, "/api")]
pub async fn delete_test(delete_test_request: DeleteTestRequest) -> Result<Test, ServerFnError> {
    let deleted_results = delete_certain_test(delete_test_request.test_identifier).await;

    match deleted_results {
        Ok(deleted) => {
            if let Some(deleted_test) = deleted {
                Ok(deleted_test)
            } else {
                Err(ServerFnError::Response(ErrorMessageTest::create(
                    TestError::TestDeleteFailure,
                )))
            }
        }
        Err(test_error) => Err(ServerFnError::Response(ErrorMessageTest::create(
            test_error,
        ))),
    }
}

#[server(EditTest, "/api")]
pub async fn edit_test(edit_test_request: EditTestRequest) -> Result<Test, ServerFnError> {
    let updated = edit_certain_test(
        edit_test_request.name,
        edit_test_request.score,
        edit_test_request.comments,
        edit_test_request.test_area,
        edit_test_request.test_identifier,
    )
    .await;

    match updated {
        Ok(updated_result) => {
            if let Some(updated_test) = updated_result {
                Ok(updated_test)
            } else {
                Err(ServerFnError::Args(ErrorMessageTest::create(
                    TestError::TestUpdateFailure,
                )))
            }
        }
        Err(test_error) => Err(ServerFnError::Args(ErrorMessageTest::create(test_error))),
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {

        use crate::app::db::database;
        use crate::app::errors::TestError;
        use chrono::{DateTime, Local};

        pub async fn retrieve_all_tests() -> Vec<Test> {

            let get_all_tests_result = database::get_all_tests().await;
            match get_all_tests_result {
                Some(found_test) => found_test,
                None => Vec::new()
            }
        }

        pub async fn add_new_test<T> (name: T, score: i32, comments: T, test_area: test_type, test_identifier: i64) -> Option<Test> where T: Into<String> {

            let current_now = Local::now();
            let current_formatted = current_now.to_string();

            let new_test = Test::new(
                name.into(),
                score,
                comments.into(),
                test_area,
                test_identifier,
                current_formatted,
            );

            database::add_test(new_test).await
        }

        pub async fn delete_certain_test(test_identifier: i64) ->
            Result<Option<Test>, TestError> {

                database::delete_test(test_identifier).await
        }

        pub async fn edit_certain_test<T>(name: T, score: i32, comments: T, test_area: test_type, test_identifier: i64) -> Result<Option<Test>, TestError> where T:Into<String> {

            database::update_test(name.into(), score, comments.into(), test_area, test_identifier).await
        }

    }
}
