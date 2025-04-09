use crate::app::models::session::{
    CreateTestSessionRequest, TestSession, UpdateTestSessionRequest,
};
use chrono::{DateTime, Utc};
use leptos::ServerFnError;

cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")]{
        use uuid::Uuid;
        use sqlx::PgPool;
        use leptos::*;
        use sqlx::prelude::*;

        pub async fn get_all_sessions(pool: &sqlx::PgPool) -> Result<Vec<TestSession>, ServerFnError> {
            let rows = sqlx::query("SELECT session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at")
                .fetch_all(pool)
                .await?;

            let sessions: Vec<TestSession> = rows
                .into_iter()
                .map(|row| {
                    let session_id: String = row.get("session_id");
                    let test_id: String = row.get("test_id");
                    let student_id: i32 = row.get("student_id");
                    let evaluator_id: String = row.get("evaluator_id");
                    let current_card_index: i32 = row.get("current_card_index");
                    let is_active: bool = row.get("is_active");
                    let created_at: DateTime<Utc> = row.get("created_at");
                    let updated_at: DateTime<Utc> = row.get("updated_at");
                    let completed_at: DateTime<Utc> = row.get("completed_at");

                    TestSession {
                        session_id,
                        test_id,
                        student_id,
                        evaluator_id,
                        current_card_index,
                        is_active,
                        created_at,
                        updated_at,
                        completed_at: Some(completed_at),
                    }
                })
                .collect();
            Ok(sessions)
        }

        pub async fn create_session(create_test_session_request: CreateTestSessionRequest, pool: &sqlx::PgPool) -> Result<TestSession, ServerFnError> {
            let session_id = Uuid::new_v4().to_string();

            let row = sqlx::query("INSERT INTO test_sessions (session_id, test_id, student_id, evaluator_id, current_card_index, is_active) VALUES ($1, $2, $3, $4, 0, true) RETURNING session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at")
                .bind(&session_id)
                .bind(&create_test_session_request.test_id)
                .bind(create_test_session_request.student_id)
                .bind(&create_test_session_request.evaluator_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let session =  TestSession {
                session_id: row.get("session_id"),
                test_id: row.get("test_id"),
                student_id: row.get("student_id"),
                evaluator_id: row.get("evaluator_id"),
                current_card_index: row.get("current_card_index"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                completed_at: row.get("completed_at"),
            };

            Ok(session)
        }

        pub async fn get_session_by_id(session_id: &str, pool: &sqlx::PgPool) -> Result<Option<TestSession>, ServerFnError> {
            let row_result = sqlx::query("SELECT session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at FROM test_sessions WHERE session_id = $1")
                .bind(session_id)
                .fetch_one(pool)
                .await;

            match row_result {
                Ok(row) => {
                    let session = TestSession {
                        session_id: row.get("session_id"),
                        test_id: row.get("test_id"),
                        student_id: row.get("student_id"),
                        evaluator_id: row.get("evaluator_id"),
                        current_card_index: row.get("current_card_index"),
                        is_active: row.get("is_active"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                        completed_at: row.get("completed_at"),
                    };

                    Ok(Some(session))
                },
                Err(sqlx::Error::RowNotFound) => Ok(None),
                Err(e) => Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }

        pub async fn update_session(update_test_session_request: UpdateTestSessionRequest, pool: &sqlx::PgPool) -> Result<TestSession, ServerFnError> {
            let now = Utc::now();


            // Based on what fields are present in the request, construct the appropriate query
            let query = if update_test_session_request.is_active == Some(false) && update_test_session_request.completed_at.is_none() {
                // Case: Completing a session without specifying completed_at
                sqlx::query("UPDATE test_sessions SET updated_at = $1, is_active = false, completed_at = $1 WHERE session_id = $2 RETURNING session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at")
                    .bind(now)
                    .bind(&update_test_session_request.session_id)
            } else if let (Some(current_card_index), Some(is_active), Some(completed_at)) = (update_test_session_request.current_card_index, update_test_session_request.is_active, update_test_session_request.completed_at) {
                // Case: Updating all fields
                sqlx::query("UPDATE test_sessions SET updated_at = $1, current_card_index = $2, is_active = $3, completed_at = $4 WHERE session_id = $5 RETURNING session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at")
                    .bind(now)
                    .bind(current_card_index)
                    .bind(is_active)
                    .bind(completed_at)
                    .bind(&update_test_session_request.session_id)
            } else if let (Some(current_card_index), Some(is_active)) = (update_test_session_request.current_card_index, update_test_session_request.is_active) {
                // Case: Updating current_card_index and is_active
                sqlx::query("UPDATE test_sessions SET updated_at = $1, current_card_index = $2, is_active = $3 WHERE session_id = $4 RETURNING session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at")
                    .bind(now)
                    .bind(current_card_index)
                    .bind(is_active)
                    .bind(&update_test_session_request.session_id)
            } else if let (Some(current_card_index), Some(completed_at)) = (update_test_session_request.current_card_index, update_test_session_request.completed_at) {
                // Case: Updating current_card_index and completed_at
                sqlx::query("UPDATE test_sessions SET updated_at = $1, current_card_index = $2, completed_at = $3 WHERE session_id = $4 RETURNING session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at")
                    .bind(now)
                    .bind(current_card_index)
                    .bind(completed_at)
                    .bind(&update_test_session_request.session_id)
            } else if let (Some(is_active), Some(completed_at)) = (update_test_session_request.is_active, update_test_session_request.completed_at) {
                // Case: Updating is_active and completed_at
                sqlx::query("UPDATE test_sessions SET updated_at = $1, is_active = $2, completed_at = $3 WHERE session_id = $4 RETURNING session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at")
                    .bind(now)
                    .bind(is_active)
                    .bind(completed_at)
                    .bind(&update_test_session_request.session_id)
            } else if let Some(current_card_index) = update_test_session_request.current_card_index {
                // Case: Updating only current_card_index
                sqlx::query("UPDATE test_sessions SET updated_at = $1, current_card_index = $2 WHERE session_id = $3 RETURNING session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at")
                    .bind(now)
                    .bind(current_card_index)
                    .bind(&update_test_session_request.session_id)
            } else if let Some(is_active) = update_test_session_request.is_active {
                // Case: Updating only is_active
                sqlx::query("UPDATE test_sessions SET updated_at = $1, is_active = $2 WHERE session_id = $3 RETURNING session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at")
                    .bind(now)
                    .bind(is_active)
                    .bind(&update_test_session_request.session_id)
            } else if let Some(completed_at) = update_test_session_request.completed_at {
                // Case: Updating only completed_at
                sqlx::query("UPDATE test_sessions SET updated_at = $1, completed_at = $2 WHERE session_id = $3 RETURNING session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at")
                    .bind(now)
                    .bind(completed_at)
                    .bind(&update_test_session_request.session_id)
            } else {
                // Case: No specific fields to update, just update the timestamp
                sqlx::query("UPDATE test_sessions SET updated_at = $1 WHERE session_id = $2 RETURNING session_id, test_id, student_id, evaluator_id, current_card_index, is_active, created_at, updated_at, completed_at")
                    .bind(now)
                    .bind(&update_test_session_request.session_id)
            };

            // Execute the query
            let row = query
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            // Construct the TestSession from the row
            let session = TestSession {
                session_id: row.get("session_id"),
                test_id: row.get("test_id"),
                student_id: row.get("student_id"),
                evaluator_id: row.get("evaluator_id"),
                current_card_index: row.get("current_card_index"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                completed_at: row.get("completed_at"),
            };

            Ok(session)
        }

        pub async fn list_sessions_by_test(test_id: String, pool: &sqlx::PgPool) -> Result<Vec<TestSession>, ServerFnError> {
            let row = sqlx::query("SELECT * FROM test_sessions WHERE test_id = $1 ORDER BY created_at DESC")
                .bind(test_id)
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let sessions: Vec<TestSession> = row
                .into_iter()
                .map(|row| {
                    let session_id: String = row.get("session_id");
                    let test_id: String = row.get("test_id");
                    let student_id: i32 = row.get("student_id");
                    let evaluator_id: String = row.get("evaluator_id");
                    let current_card_index: i32 = row.get("current_card_index");
                    let is_active: bool = row.get("is_active");
                    let created_at: DateTime<Utc> = row.get("created_at");
                    let updated_at: DateTime<Utc> = row.get("updated_at");
                    let completed_at: DateTime<Utc> = row.get("completed_at");

                    TestSession {
                        session_id,
                        test_id,
                        student_id,
                        evaluator_id,
                        current_card_index,
                        is_active,
                        created_at,
                        updated_at,
                        completed_at: Some(completed_at),
                    }
                })
                .collect();
            Ok(sessions)
        }

        pub async fn list_active_sessions_by_evaluator(evaluator_id: String, pool: &sqlx::PgPool) -> Result<Vec<TestSession>, ServerFnError> {
            let row = sqlx::query("SELECT * FROM test_sessions WHERE evaluator_id = $1 ORDER BY created_at DESC")
                .bind(evaluator_id)
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let sessions: Vec<TestSession> = row
                .into_iter()
                .map(|row| {
                    let session_id: String = row.get("session_id");
                    let test_id: String = row.get("test_id");
                    let student_id: i32 = row.get("student_id");
                    let evaluator_id: String = row.get("evaluator_id");
                    let current_card_index: i32 = row.get("current_card_index");
                    let is_active: bool = row.get("is_active");
                    let created_at: DateTime<Utc> = row.get("created_at");
                    let updated_at: DateTime<Utc> = row.get("updated_at");
                    let completed_at: DateTime<Utc> = row.get("completed_at");

                    TestSession {
                        session_id,
                        test_id,
                        student_id,
                        evaluator_id,
                        current_card_index,
                        is_active,
                        created_at,
                        updated_at,
                        completed_at: Some(completed_at),
                    }
                })
                .collect();
            Ok(sessions)
        }

        pub async fn complete_session(session_id: String, pool: &sqlx::PgPool) -> Result<TestSession, ServerFnError> {
            let now = Utc::now();

            let row = sqlx::query("UPDATE test_sessions SET is_active = false, completed_at = $1 WHERE session_id = $2 RETURNING session_id, test_id, student_id, current_card_index, is_active, created_at, updated_at, completed_at")
                .bind(now)
                .bind(session_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
            let session = TestSession {
                session_id: row.get("session_id"),
                test_id: row.get("test_id"),
                student_id: row.get("student_id"),
                evaluator_id: row.get("evaluator_id"),
                current_card_index: row.get("current_card_index"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                completed_at: row.get("completed_at"),
            };

            Ok(session)
        }
    }
}
