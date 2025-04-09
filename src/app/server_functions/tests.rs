use crate::app::models::test::BenchmarkCategory;
use crate::app::models::TestType;
use crate::app::models::{test::Test, CreateNewTestRequest, DeleteTestRequest, UpdateTestRequest};
use leptos::*;
#[cfg(feature = "ssr")]
use {
    crate::app::db::database, crate::app::db::test_database, actix_web::web, chrono::Local,
    sqlx::PgPool, std::error::Error, uuid::Uuid,
};
//this file contains a list of api functions that will be called on the server side
//lowercase functions denot functions that are server side while upper/camel case functions
//indicate Client side Objects/functions
//
#[server(GetTests, "/api")]
pub async fn get_tests() -> Result<Vec<Test>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve all tests from database");

        match test_database::get_all_tests(&pool).await {
            Ok(tests) => {
                log::info!("Successfully retrieved all tests from database");
                Ok(tests)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(GetTest, "/api")]
pub async fn get_test(test_id: String) -> Result<Test, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to retrieve test");

        match test_database::get_test(test_id, &pool).await {
            Ok(test) => {
                log::info!("Successfully retrieved test from database");
                Ok(test)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(AddTest, "/api")]
pub async fn add_test(add_test_request: CreateNewTestRequest) -> Result<Test, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to add new test to the database");

        let ID = Uuid::new_v4().to_string();
        let bufferTest = Test::new(
            add_test_request.name,
            add_test_request.score,
            add_test_request.comments,
            add_test_request.testarea,
            add_test_request.school_year,
            add_test_request.benchmark_categories,
            add_test_request.test_variant,
            add_test_request.grade_level,
            ID,
        );
        test_database::add_test(&bufferTest, &pool)
            .await
            .map_err(|e| {
                log::error!("Database error while adding test: {}", e);
                ServerFnError::new(format!("Database error: {}", e))
            })
    }
}

#[server(DeleteTest, "/api")]
pub async fn delete_test(delete_test_request: DeleteTestRequest) -> Result<Test, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to delete test");

        match test_database::delete_test(delete_test_request.test_id, &pool).await {
            Ok(deleted) => Ok(deleted),
            Err(_) => Err(ServerFnError::new("Error in deleting test")),
        }
    }
}

#[server(EditTest, "/api")]
pub async fn update_test(update_test_request: UpdateTestRequest) -> Result<Test, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to update test");

        let buffer_test = Test::new(
            update_test_request.name,
            update_test_request.score,
            update_test_request.comments,
            update_test_request.testarea,
            update_test_request.school_year,
            update_test_request.benchmark_categories,
            update_test_request.test_variant,
            update_test_request.grade_level,
            update_test_request.test_id,
        );

        match test_database::update_test(&buffer_test, &pool).await {
            Ok(Some(updated_test)) => Ok(updated_test),
            Ok(None) => Err(ServerFnError::new(format!(
                "A None value was returned instead of an updated test"
            ))),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update student: {}",
                e
            ))),
        }
    }
}

#[server(ScoreOverride, "/api")]
pub async fn score_overrider(test_id: String, score: i32) -> Result<Test, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to modify score for a test");

        match test_database::score_override(test_id, score, &pool).await {
            Ok(updated_test) => Ok(updated_test),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update student: {}",
                e
            ))),
        }
    }
}

/*cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {

        use crate::app::errors::TestError;

        pub async fn retrieve_all_tests() -> Vec<Test> {
            let pool = use_context::<PgPool>().expect("Server could not find pool");

            let get_all_tests_result = database::get_all_tests(&pool).await.unwrap();

            get_all_tests_result
        }

        pub async fn add_new_test<T> (name: T, score: i32, comments: T, testarea: TestType) -> Option<Test> where T: Into<String> {

            let mut buffer = Uuid::encode_buffer();
            let uuid = Uuid::new_v4().simple().encode_lower(&mut buffer);

            let pool = use_context::<PgPool>().expect("Server could not find pool");

            let new_test = Test::new(
                name.into(),
                score,
                comments.into(),
                testarea,
                String::from(uuid),
            );

            let added_test = database::add_test(&new_test, &pool).await.unwrap();
            Some(added_test)
        }

        pub async fn delete_certain_test<T> (test_id: T) ->
            Result<(), ServerFnError> where T: Into<String> {
            let pool = use_context::<PgPool>().expect("Server could not find the pool");

                database::delete_test(test_id.into(), &pool).await
        }

        pub async fn update_certain_test<T>(name: T, score: i32, comments: T, testarea: TestType, test_id: T) -> Result<Option<Test>, sqlx::Error> where T:Into<String> {
            let pool = use_context::<PgPool>().expect("Server could not connect to pool");

            let updated_test = Test::new(
                name.into(),
                score,
                comments.into(),
                testarea,
                test_id.into(),
            );

            database::update_test(&updated_test, &pool).await
        }

    }
}*/
