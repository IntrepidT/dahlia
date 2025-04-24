use crate::app::models::websocket_session::{
    CreateSessionRequest, Session, SessionStatus, SessionSummary, SessionType,
};
use chrono::{DateTime, Utc};
use leptos::*;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use {
    crate::app::db::websocket_session_database, actix_web::web, leptos_actix::extract,
    sqlx::PgPool, std::error::Error,
};

#[server(ListActiveSessions, "/api")]
pub async fn list_active_sessions() -> Result<Vec<SessionSummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Retrieving active sessions from database");

        // First, clean up any expired sessions
        if let Err(e) = websocket_session_database::cleanup_inactive_sessions(&pool).await {
            log::warn!("Failed to clean up inactive sessions: {}", e);
        }

        // Then get active sessions
        let sessions = websocket_session_database::get_active_sessions(&pool).await?;

        // Convert to summary objects that are safer to send to the client
        let summaries = sessions.into_iter().map(SessionSummary::from).collect();

        Ok(summaries)
    }
}

#[server(GetActiveTestSessions, "/api")]
pub async fn get_active_test_sessions() -> Result<Vec<SessionSummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Retrieving active test sessions from database");

        let sessions = websocket_session_database::get_active_test_sessions(&pool).await?;

        let summaries = sessions.into_iter().map(SessionSummary::from).collect();

        Ok(summaries)
    }
}

#[server(GetSession, "/api")]
pub async fn get_session(session_id: String) -> Result<Option<SessionSummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let uuid = Uuid::parse_str(&session_id)
            .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

        log::info!("Retrieving session with ID: {}", uuid);

        let session = websocket_session_database::get_session(uuid, &pool).await?;

        Ok(session.map(SessionSummary::from))
    }
}

#[server(GetTestSessionsByTestId, "/api")]
pub async fn get_test_sessions_by_test_id(
    test_id: String,
) -> Result<Vec<SessionSummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Retrieving test sessions for test ID: {}", test_id);

        let sessions = websocket_session_database::get_sessions_by_test_id(&test_id, &pool).await?;

        let summaries = sessions.into_iter().map(SessionSummary::from).collect();

        Ok(summaries)
    }
}

#[server(CreateSession, "/api")]
pub async fn create_session(
    request: CreateSessionRequest,
) -> Result<SessionSummary, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Creating new session with name: {}", request.name);

        let session_type = request.session_type.unwrap_or(SessionType::Chat);

        let mut session = Session::new(
            request.name,
            request.description,
            None, // You might want to store the user ID here if you have authentication
            session_type,
            request.test_id,
        );

        // Set optional fields
        if let Some(max_users) = request.max_users {
            session.max_users = max_users;
        }

        if let Some(is_private) = request.is_private {
            session.is_private = is_private;
        }

        if let Some(password) = &request.password {
            session.password_required = true;
            // In a real app, you would hash the password here
        }

        if let Some(metadata) = request.metadata {
            session.metadata = Some(metadata);
        }

        let created_session = websocket_session_database::create_session(&session, &pool).await?;

        Ok(SessionSummary::from(created_session))
    }
}

#[server(StartTestSession, "/api")]
pub async fn start_test_session(
    session_id: String,
    scheduled_end_time: Option<DateTime<Utc>>,
) -> Result<SessionSummary, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let uuid = Uuid::parse_str(&session_id)
            .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

        log::info!("Starting test session: {}", uuid);

        let session_opt = websocket_session_database::get_session(uuid, &pool).await?;

        if let Some(mut session) = session_opt {
            if session.session_type != SessionType::Test {
                return Err(ServerFnError::new("Session is not a test session"));
            }

            let start_time = Utc::now();
            session.start_time = Some(start_time);
            session.end_time = scheduled_end_time;

            let updated_session = websocket_session_database::update_test_session_times(
                uuid,
                Some(start_time),
                scheduled_end_time,
                &pool,
            )
            .await?;

            Ok(SessionSummary::from(updated_session))
        } else {
            Err(ServerFnError::new("Session not found"))
        }
    }
}

#[server(JoinSession, "/api")]
pub async fn join_session(
    session_id: String,
    password: Option<String>,
) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let uuid = Uuid::parse_str(&session_id)
            .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

        log::info!("User attempting to join session: {}", uuid);

        // Get the session to check if it exists and if it requires a password
        let session_opt = websocket_session_database::get_session(uuid, &pool).await?;

        match session_opt {
            Some(session) => {
                // Check if session is full
                if session.max_users > 0 && session.current_users >= session.max_users {
                    return Err(ServerFnError::new("Session is full"));
                }

                // Check if session requires password
                if session.password_required {
                    // In a real app, you would verify the password hash here
                    if password.is_none() {
                        return Err(ServerFnError::new("Password required for this session"));
                    }

                    // Simple example - in a real app you would compare hashed passwords
                    // if password.unwrap() != "correct_password" {
                    //     return Err(ServerFnError::new("Incorrect password"));
                    // }
                }

                // Increment user count
                websocket_session_database::update_session_user_count(uuid, true, &pool).await?;

                Ok(true)
            }
            None => Err(ServerFnError::new("Session not found")),
        }
    }
}

#[server(LeaveSession, "/api")]
pub async fn leave_session(session_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let uuid = Uuid::parse_str(&session_id)
            .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

        log::info!("User leaving session: {}", uuid);

        websocket_session_database::update_session_user_count(uuid, false, &pool).await?;

        Ok(())
    }
}

#[server(EndTestSession, "/api")]
pub async fn end_test_session(session_id: String) -> Result<SessionSummary, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let uuid = Uuid::parse_str(&session_id)
            .map_err(|e| ServerFnError::new(format!("Invalid Uuid: {}", e)))?;

        log::info!("Ending test session: {}", uuid);

        let session_opt = websocket_session_database::get_session(uuid, &pool).await?;

        if let Some(session) = session_opt {
            if session.session_type != SessionType::Test {
                return Err(ServerFnError::new("Session is not a test session"));
            }

            let end_time = Utc::now();

            let updated_session = websocket_session_database::update_test_session_times(
                uuid,
                session.start_time,
                Some(end_time),
                &pool,
            )
            .await?;

            websocket_session_database::update_session_status(uuid, SessionStatus::Inactive, &pool)
                .await?;

            Ok(SessionSummary::from(updated_session))
        } else {
            Err(ServerFnError::new("Session not found"))
        }
    }
}

#[server(CloseSession, "/api")]
pub async fn close_session(session_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let uuid = Uuid::parse_str(&session_id)
            .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

        log::info!("Closing session: {}", uuid);

        websocket_session_database::update_session_status(uuid, SessionStatus::Inactive, &pool)
            .await?;

        Ok(())
    }
}
