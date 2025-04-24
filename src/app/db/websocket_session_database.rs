use leptos::ServerFnError;
use uuid::Uuid;

use crate::app::{models::websocket_session::SessionType, server_functions::tests::update_test};

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")]{
        use crate::app::models::websocket_session::Session;
        use crate::app::models::websocket_session::SessionStatus;
        use log::{debug, error, info, warn};
        use chrono::{DateTime, Utc};
        use leptos::*;
        use sqlx::PgPool;
        use sqlx::prelude::*;
        use sqlx::types::JsonValue;

        /// Retrieves all active sessions from the database
        pub async fn get_active_sessions(pool: &PgPool) -> Result<Vec<Session>, ServerFnError> {
            let rows = sqlx::query(
                "SELECT id, name, description, created_at, last_active, owner_id,
                status, max_users, current_users, is_private, password_required, metadata,
                session_type, test_id, start_time, end_time
                FROM websocket_sessions WHERE status = 'active' ORDER BY last_active DESC"
            )
            .fetch_all(pool)
            .await?;

            let sessions: Vec<Session> = rows
                .into_iter()
                .map(|row| {
                    let id: Uuid = row.get("id");
                    let name: String = row.get("name");
                    let description: Option<String> = row.get("description");
                    let created_at: DateTime<Utc> = row.get("created_at");
                    let last_active: DateTime<Utc> = row.get("last_active");
                    let owner_id: Option<Uuid> = row.get("owner_id");
                    let status: SessionStatus = row.get("status");
                    let max_users: i32 = row.get("max_users");
                    let current_users: i32 = row.get("current_users");
                    let is_private: bool = row.get("is_private");
                    let password_required: bool = row.get("password_required");
                    let metadata: Option<JsonValue> = row.get("metadata");
                    let session_type: SessionType = row.get("session_type");
                    let test_id: Option<String> = row.get("test_id");
                    let start_time: Option<DateTime<Utc>> = row.get("start_time");
                    let end_time: Option<DateTime<Utc>> = row.get("end_time");

                    Session {
                        id,
                        name,
                        description,
                        created_at,
                        last_active,
                        owner_id,
                        status,
                        max_users,
                        current_users,
                        is_private,
                        password_required,
                        metadata,
                        session_type,
                        test_id,
                        start_time,
                        end_time,
                    }
                })
                .collect();

            Ok(sessions)
        }

        ///Get all active test sessions
        pub async fn get_active_test_sessions(pool: &PgPool) -> Result<Vec<Session>, ServerFnError> {
            let rows = sqlx::query(
                "SELECT id, name, description, created_at, last_active, owner_id,
                status, max_users, current_users, is_private, password_required, metadata, 
                session_type, test_id, start_time, end_time
                FROM websocket_sessions
                WHERE status = 'active' AND session_type = 'test'
                ORDER BY last_active DESC"
            )
            .fetch_all(pool)
            .await?;

            let sessions: Vec<Session> = rows
                .into_iter()
                .map(|row| {
                    let id: Uuid = row.get("id");
                    let name: String = row.get("name");
                    let description: Option<String> = row.get("description");
                    let created_at: DateTime<Utc> = row.get("created_at");
                    let last_active: DateTime<Utc> = row.get("last_active");
                    let owner_id: Option<Uuid> = row.get("owner_id");
                    let status: SessionStatus = row.get("status");
                    let max_users: i32 = row.get("max_users");
                    let current_users: i32 = row.get("current_users");
                    let is_private: bool = row.get("is_private");
                    let password_required: bool = row.get("password_required");
                    let metadata: Option<JsonValue> = row.get("metadata");
                    let session_type: SessionType = row.get("session_type");
                    let test_id: Option<String> = row.get("test_id");
                    let start_time: Option<DateTime<Utc>> = row.get("start_time");
                    let end_time: Option<DateTime<Utc>> = row.get("end_time");

                    Session {
                        id,
                        name,
                        description,
                        created_at,
                        last_active,
                        owner_id,
                        status,
                        max_users,
                        current_users,
                        is_private,
                        password_required,
                        metadata,
                        session_type,
                        test_id,
                        start_time,
                        end_time,
                    }
                })
                .collect();

            Ok(sessions)
        }

        /// Retrieves a specific session by ID
        pub async fn get_session(session_id: Uuid, pool: &PgPool) -> Result<Option<Session>, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, name, description, created_at, last_active, owner_id,
                status, max_users, current_users, is_private, password_required, metadata,
                session_type, test_id, start_time, end_time
                FROM websocket_sessions WHERE id = $1"
            )
            .bind(session_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            match row {
                Some(row) => {
                    let session = Session {
                        id: row.get("id"),
                        name: row.get("name"),
                        description: row.get("description"),
                        created_at: row.get("created_at"),
                        last_active: row.get("last_active"),
                        owner_id: row.get("owner_id"),
                        status: row.get("status"),
                        max_users: row.get("max_users"),
                        current_users: row.get("current_users"),
                        is_private: row.get("is_private"),
                        password_required: row.get("password_required"),
                        metadata: row.get("metadata"),
                        session_type: row.get("session_type"),
                        test_id: row.get("test_id"),
                        start_time: row.get("start_time"),
                        end_time: row.get("end_time"),
                    };
                    Ok(Some(session))
                },
                None => Ok(None),
            }
        }

        ///Get sessions by test ID
        pub async fn get_sessions_by_test_id(test_id: &str, pool: &PgPool) -> Result<Vec<Session>, ServerFnError> {
            let rows = sqlx::query(
                "SELECT id, name, description, created_at, last_active, owner_id,
                status, max_users, current_users, is_private, password_required, metadata,
                session_type, test_id, start_time, end_time
                FROM websocket_sessions
                WHERE test_id = $1 
                ORDER BY created_at DESC"
            )
            .bind(test_id)
            .fetch_all(pool)
            .await?;

            let sessions: Vec<Session> = rows
                .into_iter()
                .map(|row| {
                    let id: Uuid = row.get("id");
                    let name: String = row.get("name");
                    let description: Option<String> = row.get("description");
                    let created_at: DateTime<Utc> = row.get("created_at");
                    let last_active: DateTime<Utc> = row.get("last_active");
                    let owner_id: Option<Uuid> = row.get("owner_id");
                    let status: SessionStatus = row.get("status");
                    let max_users: i32 = row.get("max_users");
                    let current_users: i32 = row.get("current_users");
                    let is_private: bool = row.get("is_private");
                    let password_required: bool = row.get("password_required");
                    let metadata: Option<JsonValue> = row.get("metadata");
                    let session_type: SessionType = row.get("session_type");
                    let test_id: Option<String> = row.get("test_id");
                    let start_time: Option<DateTime<Utc>> = row.get("start_time");
                    let end_time: Option<DateTime<Utc>> = row.get("end_time");

                    Session {
                        id,
                        name,
                        description,
                        created_at,
                        last_active,
                        owner_id,
                        status,
                        max_users,
                        current_users,
                        is_private,
                        password_required,
                        metadata,
                        session_type,
                        test_id,
                        start_time,
                        end_time,
                    }
                })
                .collect();

            Ok(sessions)
        }

        /// Creates a new session
        pub async fn create_session(session: &Session, pool: &PgPool) -> Result<Session, ServerFnError> {
            let row = sqlx::query(
                "INSERT INTO websocket_sessions
                (id, name, description, owner_id, status, max_users, is_private, password_required, password_hash, metadata, session_type, test_id, start_time, end_time) 
                VALUES ($1, $2, $3, $4, $5::session_status_enum, $6, $7, $8, $9, $10, $11::session_type_enum, $12, $13, $14) 
                RETURNING id, name, description, created_at, last_active, owner_id, 
                status, max_users, current_users, is_private, password_required, metadata, session_type, test_id, start_time, end_time"
            )
            .bind(session.id)
            .bind(&session.name)
            .bind(&session.description)
            .bind(session.owner_id)
            .bind(&session.status.to_string())
            .bind(session.max_users)
            .bind(session.is_private)
            .bind(session.password_required)
            .bind(None::<String>) // password_hash - would implement proper hashing in production
            .bind(&session.metadata)
            .bind(&session.session_type.to_string())
            .bind(&session.test_id)
            .bind(session.start_time)
            .bind(session.end_time)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let session = Session {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                created_at: row.get("created_at"),
                last_active: row.get("last_active"),
                owner_id: row.get("owner_id"),
                status: row.get("status"),
                max_users: row.get("max_users"),
                current_users: row.get("current_users"),
                is_private: row.get("is_private"),
                password_required: row.get("password_required"),
                metadata: row.get("metadata"),
                session_type: row.get("session_type"),
                test_id: row.get("test_id"),
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
            };

            Ok(session)
        }

        /// Updates session user count
        pub async fn update_session_user_count(session_id: Uuid, increment: bool, pool: &PgPool) -> Result<(), ServerFnError> {
            let sql = if increment {
                "UPDATE websocket_sessions SET current_users = current_users + 1, last_active = NOW() WHERE id = $1"
            } else {
                "UPDATE websocket_sessions SET current_users = GREATEST(current_users - 1, 0), last_active = NOW() WHERE id = $1"
            };

            sqlx::query(sql)
                .bind(session_id)
                .execute(pool)
                .await?;

            Ok(())
        }

        /// Updates a session's status
        pub async fn update_session_status(session_id: Uuid, status: SessionStatus, pool: &PgPool) -> Result<(), ServerFnError> {
            sqlx::query("UPDATE websocket_sessions SET status = $1::session_status_enum, last_active = NOW() WHERE id = $2")
                .bind(status)
                .bind(session_id)
                .execute(pool)
                .await?;

            Ok(())
        }

        ///Updates a test session's start and end times
        pub async fn update_test_session_times(
            session_id: Uuid,
            start_time: Option<DateTime<Utc>>,
            end_time: Option<DateTime<Utc>>,
            pool: &PgPool
        ) -> Result<Session, ServerFnError> {
            let update_result = sqlx::query(
                "UPDATE websocket_sessions
                SET start_time = $1, end_time = $2, last_active = NOW()
                WHERE id =  $3 
                RETURNING id, name, description, created_at, last_active, owner_id, 
                status, max_users, current_users, is_private, password_required, metadata,
                session_type, test_id, start_time, end_time"
            )
            .bind(start_time)
            .bind(end_time)
            .bind(session_id)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let session = Session {
                id: update_result.get("id"),
                name: update_result.get("name"),
                description: update_result.get("description"),
                created_at: update_result.get("created_at"),
                last_active: update_result.get("last_active"),
                owner_id: update_result.get("owner_id"),
                status: update_result.get("status"),
                max_users: update_result.get("max_users"),
                current_users: update_result.get("current_users"),
                is_private: update_result.get("is_private"),
                password_required: update_result.get("password_required"),
                metadata: update_result.get("metadata"),
                session_type: update_result.get("session_type"),
                test_id: update_result.get("test_id"),
                start_time: update_result.get("start_time"),
                end_time: update_result.get("end_time"),
            };

            Ok(session)
        }

        /// Deletes a session
        pub async fn delete_session(session_id: Uuid, pool: &PgPool) -> Result<(), ServerFnError> {
            sqlx::query("DELETE FROM websocket_sessions WHERE id = $1")
                .bind(session_id)
                .execute(pool)
                .await?;

            Ok(())
        }

        //Auto-expire test session that have passed their end times
        pub async fn expire_completed_test_sessions(pool: &PgPool) -> Result<(), ServerFnError> {
            sqlx::query(
                "UPDATE websocket_sessions
                SET status = 'expired'::session_status_enum
                WHERE status = 'active'::session_status_enum
                AND session_type = 'test'::session_type_enum
                AND end_time IS NOT NULL
                AND end_time < NOW()"
            )
            .execute(pool)
            .await?;

            Ok(())
        }

        //Cleanup expired or empty session (for test and chat session)
        pub async fn cleanup_inactive_sessions(pool: &PgPool) -> Result<(), ServerFnError> {
            sqlx::query(
                "UPDATE websocket_sessions SET status = 'expired'::session_status_enum
                WHERE status = 'active'::session_status_enum
                AND last_active < NOW() - INTERVAL '24 hours'"
            )
            .execute(pool)
            .await?;

            sqlx::query(
                "UPDATE websocket_sessions SET status = 'inactive'::session_status_enum
                WHERE status = 'active'::session_status_enum
                AND current_users = 0
                AND last_active < NOW() - INTERVAL '1 hour'"
            )
            .execute(pool)
            .await?;

            sqlx::query(
                "Update websocket_sessions
                SET status = 'expired'::session_status_enum
                WHERE status = 'active'::session_status_enum
                AND session_type = 'test'::session_type_enum
                AND end_time IS NOT NULL
                AND end_time < NOW()"
            )
            .execute(pool)
            .await?;

            Ok(())
        }
    }
}
