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
                session_type, test_id, start_time, end_time, teacher_id
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
                    let teacher_id: Option<i32> = row.get("teacher_id");

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
                        teacher_id,
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
                session_type, test_id, start_time, end_time, teacher_id
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
                    let teacher_id: Option<i32> = row.get("teacher_id");

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
                        teacher_id,
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
                session_type, test_id, start_time, end_time, teacher_id
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
                        teacher_id: row.get("teacher_id"),
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
                session_type, test_id, start_time, end_time, teacher_id
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
                    let teacher_id: Option<i32> = row.get("teacher_id");

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
                        teacher_id,
                    }
                })
                .collect();

            Ok(sessions)
        }

        /// Creates a new session
        pub async fn create_session(session: &Session, pool: &PgPool) -> Result<Session, ServerFnError> {
        let row = sqlx::query(
                "INSERT INTO websocket_sessions
                (id, name, description, owner_id, status, max_users, is_private, password_required, password_hash, metadata, session_type, test_id, start_time, end_time, teacher_id) 
                VALUES ($1, $2, $3, $4, $5::session_status_enum, $6, $7, $8, $9, $10, $11::session_type_enum, $12, $13, $14, $15) 
                RETURNING id, name, description, created_at, last_active, owner_id, 
                status, max_users, current_users, is_private, password_required, metadata, session_type, test_id, start_time, end_time, teacher_id"
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
            .bind(session.teacher_id)
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
                teacher_id: row.get("teacher_id"),
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
                session_type, test_id, start_time, end_time, teacher_id"
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
                teacher_id: update_result.get("teacher_id"),
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
            // 1. Clean up sessions where teacher has been inactive for 10 seconds
            sqlx::query(
                "UPDATE websocket_sessions
                 SET status = 'inactive'::session_status_enum,
                     teacher_id = NULL
                 WHERE status = 'active'::session_status_enum
                 AND teacher_id IS NOT NULL
                 AND last_active < NOW() - INTERVAL '10 seconds'"
            )
            .execute(pool)
            .await?;

            // 2. Clean up duplicate sessions for same teacher (keep most recent)
            sqlx::query(
                "UPDATE websocket_sessions s1
                 SET status = 'inactive'::session_status_enum,
                     teacher_id = NULL
                 WHERE s1.status = 'active'::session_status_enum
                 AND s1.teacher_id IS NOT NULL
                 AND EXISTS (
                     SELECT 1 FROM websocket_sessions s2
                     WHERE s2.teacher_id = s1.teacher_id
                     AND s2.id != s1.id
                     AND s2.status = 'active'::session_status_enum
                     AND s2.last_active > s1.last_active
                 )"
            )
            .execute(pool)
            .await?;
            // 3. Clean up sessions where teacher has been marked as disconnected (more aggressive timing)
            sqlx::query(
                "UPDATE websocket_sessions
                 SET status = 'inactive'::session_status_enum,
                     teacher_id = NULL
                 WHERE status = 'active'::session_status_enum
                 AND teacher_id IS NOT NULL
                 AND last_active < NOW() - INTERVAL '30 seconds'"  // REDUCED from 2 minutes
            )
            .execute(pool)
            .await?;

            // 4. Mark empty sessions as inactive after 5 minutes (unchanged)
            sqlx::query(
                "UPDATE websocket_sessions
                 SET status = 'inactive'::session_status_enum
                 WHERE status = 'active'::session_status_enum
                 AND current_users = 0
                 AND last_active < NOW() - INTERVAL '5 minutes'"
            )
            .execute(pool)
            .await?;

            // 5. Mark old sessions as expired after 2 hours (unchanged)
            sqlx::query(
                "UPDATE websocket_sessions
                 SET status = 'expired'::session_status_enum
                 WHERE status = 'active'::session_status_enum
                 AND last_active < NOW() - INTERVAL '2 hours'"
            )
            .execute(pool)
            .await?;

            // 6. Handle completed test sessions
            expire_completed_test_sessions(pool).await?;

            Ok(())
        }

        pub async fn assign_teacher_to_session(session_id: Uuid, teacher_id: i32, pool: &PgPool) -> Result<Session, ServerFnError> {
            let row = sqlx::query(
                "UPDATE websocket_sessions
                 SET teacher_id = $1, last_active = NOW() 
                 WHERE id = $2 
                 RETURNING id, name, description, created_at, last_active, owner_id,
                 status, max_users, current_users, is_private, password_required, metadata,
                 session_type, test_id, start_time, end_time, teacher_id"
            )
            .bind(teacher_id)
            .bind(session_id)
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
                teacher_id: row.get("teacher_id"),
            };

            Ok(session)
        }

        /// Gets the active session for a specific teacher
        pub async fn get_teacher_active_session(teacher_id: i32, pool: &PgPool) -> Result<Option<Session>, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, name, description, created_at, last_active, owner_id,
                 status, max_users, current_users, is_private, password_required, metadata,
                 session_type, test_id, start_time, end_time, teacher_id
                 FROM websocket_sessions 
                 WHERE teacher_id = $1 AND end_time IS NULL AND status = 'active'::session_status_enum"
            )
            .bind(teacher_id)
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
                        teacher_id: row.get("teacher_id"),
                    };
                    Ok(Some(session))
                },
                None => Ok(None),
            }
        }

        /// Releases a teacher from a session (sets teacher_id to NULL)
        pub async fn release_teacher_from_session(session_id: Uuid, pool: &PgPool) -> Result<(), ServerFnError> {
            sqlx::query(
                "UPDATE websocket_sessions
                 SET teacher_id = NULL, last_active = NOW() 
                 WHERE id = $1"
            )
            .bind(session_id)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            Ok(())
        }

        /// Checks if a teacher can access a specific test (no other teacher is currently active)
        pub async fn check_teacher_test_access(test_id: &str, teacher_id: i32, pool: &PgPool) -> Result<bool, ServerFnError> {
            // FIRST: Clean up any stale sessions
            cleanup_inactive_sessions(pool).await?;

            // SECOND: Check for conflicting active sessions
            let conflicting_sessions = sqlx::query(
                "SELECT COUNT(*) as count
                 FROM websocket_sessions 
                 WHERE test_id = $1 
                 AND teacher_id IS NOT NULL 
                 AND teacher_id != $2 
                 AND end_time IS NULL 
                 AND status = 'active'::session_status_enum
                 AND last_active > NOW() - INTERVAL '1 minute'"  // Only consider recently active sessions
            )
            .bind(test_id)
            .bind(teacher_id)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let count: i64 = conflicting_sessions.get("count");
            Ok(count == 0)
        }

        /// Immediately cleanup a teacher's session when they disconnect
        pub async fn cleanup_teacher_session(teacher_id: i32, pool: &PgPool) -> Result<(), ServerFnError> {
            info!("Starting enhanced cleanup for teacher {}", teacher_id);

            // Get the teacher's active session first for logging
            if let Some(session) = get_teacher_active_session(teacher_id, pool).await? {
                info!("Found active session {} for teacher {}, cleaning up...", session.id, teacher_id);

                // Update session timestamp to current time to prevent race conditions
                sqlx::query(
                    "UPDATE websocket_sessions
                     SET last_active = NOW() 
                     WHERE id = $1"
                )
                .bind(session.id)
                .execute(pool)
                .await?;

                // Set session to inactive
                update_session_status(session.id, SessionStatus::Inactive, pool).await?;

                // Release teacher from session
                release_teacher_from_session(session.id, pool).await?;

                info!("Successfully cleaned up session {} for teacher {}", session.id, teacher_id);
            } else {
                info!("No active session found for teacher {} during cleanup", teacher_id);
            }

            // Also run general cleanup to catch any other stale sessions
            cleanup_inactive_sessions(pool).await?;

            Ok(())
        }
    }
}
