use leptos::ServerFnError;
use uuid::Uuid;

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
                status, max_users, current_users, is_private, password_required, metadata 
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
                    }
                })
                .collect();

            Ok(sessions)
        }

        /// Retrieves a specific session by ID
        pub async fn get_session(session_id: Uuid, pool: &PgPool) -> Result<Option<Session>, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, name, description, created_at, last_active, owner_id,
                status, max_users, current_users, is_private, password_required, metadata 
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
                    };
                    Ok(Some(session))
                },
                None => Ok(None),
            }
        }

        /// Creates a new session
        pub async fn create_session(session: &Session, pool: &PgPool) -> Result<Session, ServerFnError> {
            let row = sqlx::query(
                "INSERT INTO websocket_sessions
                (id, name, description, owner_id, status, max_users, is_private, password_required, password_hash, metadata) 
                VALUES ($1, $2, $3, $4, $5::session_status_enum, $6, $7, $8, $9, $10) 
                RETURNING id, name, description, created_at, last_active, owner_id, 
                status, max_users, current_users, is_private, password_required, metadata"
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

        /// Deletes a session
        pub async fn delete_session(session_id: Uuid, pool: &PgPool) -> Result<(), ServerFnError> {
            sqlx::query("DELETE FROM websocket_sessions WHERE id = $1")
                .bind(session_id)
                .execute(pool)
                .await?;

            Ok(())
        }

        /// Cleanup expired or empty sessions
        pub async fn cleanup_inactive_sessions(pool: &PgPool) -> Result<(), ServerFnError> {
            // Mark sessions as expired if they've been inactive for more than 24 hours
            sqlx::query(
                "UPDATE websocket_sessions SET status = 'expired'::session_status_enum
                WHERE status = 'active'::session_status_enum 
                AND last_active < NOW() - INTERVAL '24 hours'"
            )
            .execute(pool)
            .await?;

            // Mark sessions as inactive if they have 0 users but are still marked as active
            sqlx::query(
                "UPDATE websocket_sessions SET status = 'inactive'::session_status_enum
                WHERE status = 'active'::session_status_enum 
                AND current_users = 0 
                AND last_active < NOW() - INTERVAL '1 hour'"
            )
            .execute(pool)
            .await?;

            Ok(())
        }
    }
}
