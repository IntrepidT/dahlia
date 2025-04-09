use crate::app::models::session::{
    CreateTestSessionRequest, TestSession, UpdateTestSessionRequest,
};
use chrono::{DateTime, Utc};
use leptos::*;
use uuid::Uuid;
#[cfg(feature = "ssr")]
use {
    crate::app::db::session_database, actix_web::web, chrono::Local, sqlx::PgPool,
    std::error::Error,
};

#[server(GetSessions, "/api")]
pub async fn get_sessions() -> Result<Vec<TestSession>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Getting list of sessions");

        match session_database::get_all_sessions(&pool).await {
            Ok(sessions) => {
                log::info!("Successfully retrieved all sessions from database");
                Ok(sessions)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(CreateTestSession, "/api")]
pub async fn create_test_session(
    test_id: String,
    student_id: i32,
    evaluator_id: String,
) -> Result<TestSession, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;
        log::info!("Creating new session");

        let create_session_request = CreateTestSessionRequest {
            test_id,
            student_id,
            evaluator_id,
        };

        match session_database::create_session(create_session_request, &pool).await {
            Ok(session) => Ok(session),
            Err(e) => {
                log::error!("Error creating test session: {}", e);
                Err(ServerFnError::new(format!(
                    "Failed to create session: {}",
                    e
                )))
            }
        }
    }
}

#[server(GetSession, "/api")]
pub async fn get_session(session_id: String) -> Result<Option<TestSession>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to get session");

        match session_database::get_session_by_id(&session_id, &pool).await {
            Ok(Some(session)) => Ok(Some(session)),
            Ok(None) => Ok(None),
            Err(e) => {
                log::error!("Error fetching test session: {}", e);
                Err(ServerFnError::new(format!(
                    "Failed to fetch session: {}",
                    e
                )))
            }
        }
    }
}

#[server(UpdateSessionState, "/api")]
pub async fn update_session_state(
    session_id: String,
    current_card_index: Option<i32>,
    is_active: Option<bool>,
    completed_at: Option<DateTime<Utc>>,
) -> Result<TestSession, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let update_session_request = UpdateTestSessionRequest {
            session_id,
            current_card_index,
            is_active,
            completed_at,
        };

        log::info!("Attempting to update session state");

        match session_database::update_session(update_session_request, &pool).await {
            Ok(session) => Ok(session),
            Err(e) => {
                log::error!("Error updating test session: {}", e);
                Err(ServerFnError::new(format!(
                    "Failed to update session: {}",
                    e
                )))
            }
        }
    }
}

#[server(ListSessionsByTest, "/api")]
pub async fn list_sessions_for_test(test_id: String) -> Result<Vec<TestSession>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to gather sessions for a specific test");

        match session_database::list_sessions_by_test(test_id, &pool).await {
            Ok(sessions) => Ok(sessions),
            Err(e) => {
                log::error!("Error listing test sessions: {}", e);
                Err(ServerFnError::new(format!(
                    "Failed to list sessions: {}",
                    e
                )))
            }
        }
    }
}

#[server(ListActiveSessionsByEvaluator, "/api")]
pub async fn list_active_sessions_for_evaluator(
    evaluator_id: String,
) -> Result<Vec<TestSession>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to gather active sessions for evaluator");

        match session_database::list_active_sessions_by_evaluator(evaluator_id, &pool).await {
            Ok(sessions) => Ok(sessions),
            Err(e) => {
                log::error!("Error listing active sessions: {}", e);
                Err(ServerFnError::new(format!(
                    "Failed to list sessions: {}",
                    e
                )))
            }
        }
    }
}

#[server(CompleteTestSession, "/api")]
pub async fn complete_test_session(session_id: String) -> Result<TestSession, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to complete a session");

        match session_database::complete_session(session_id, &pool).await {
            Ok(session) => Ok(session),
            Err(e) => {
                log::error!("Error completing test session: {}", e);
                Err(ServerFnError::new(format!(
                    "Failed to complete session: {}",
                    e
                )))
            }
        }
    }
}
