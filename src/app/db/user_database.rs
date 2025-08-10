use leptos::prelude::*;
cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {

        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
            Argon2,
        };
        use sqlx::{Pool, Postgres};
        use crate::app::models::user::{SessionUser, User, UserRole};
        use sqlx::Row;
        use chrono::{DateTime, Utc};
        use crate::app::models::user::AccountStatus;
        use crate::app::models::setting_data::{UserSettings, UserSettingsUpdate};

        // Hash a password
        pub fn hash_password(password: &str) -> Result<String, ServerFnError> {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            argon2
                .hash_password(password.as_bytes(), &salt)
                .map(|hash| hash.to_string())
                .map_err(|e| ServerFnError::new(format!("Password hashing error: {}", e)))
        }

        // Verify a user's password
        pub fn verify_password(password: &str, password_hash: &str) -> bool {
            let parsed_hash = match PasswordHash::new(password_hash) {
                Ok(hash) => hash,
                Err(_) => return false,
            };

            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        }

        // Create a new user - returns FULL User struct (for registration flow)
        pub async fn create_user(
            pool: &Pool<Postgres>,
            username: String,
            email: String,
            password: String,
            role: UserRole,
        ) -> Result<User, ServerFnError> {
            let password_hash = hash_password(&password)?;

            let row = sqlx::query(
                "INSERT INTO users (username, email, password_hash, role, account_status, email_verified, phone_verified)
                 VALUES ($1, $2, $3, $4, 'pending', false, false)
                 RETURNING id, username, email, password_hash, role, password_salt, account_status::text, email_verified, phone_number, phone_verified, display_name, first_name, last_name"
            )
                .bind(&username)
                .bind(&email)
                .bind(password_hash)
                .bind(&role)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
                password_salt: row.try_get("password_salt").unwrap_or(None),
                account_status: AccountStatus::from_str(row.get("account_status")),
                email_verified: row.get("email_verified"),
                phone_number: row.try_get("phone_number").unwrap_or(None),
                phone_verified: row.get("phone_verified"),
                display_name: row.try_get("display_name").unwrap_or(None),
                first_name: row.try_get("first_name").unwrap_or(None),
                last_name: row.try_get("last_name").unwrap_or(None),
            };

            Ok(user)
        }

        // Get all users (admin function) - returns FULL User structs
        pub async fn get_all_users(pool: &sqlx::PgPool) -> Result<Vec<User>, ServerFnError> {
            let rows = sqlx::query("SELECT id, username, email, password_hash, role, password_salt, account_status::text, email_verified, phone_number, phone_verified, display_name, first_name, last_name FROM users ORDER BY id ASC")
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let users: Vec<User> = rows
                .into_iter()
                .map(|row| {
                    User {
                        id: row.get("id"),
                        username: row.get("username"),
                        email: row.get("email"),
                        password_hash: row.get("password_hash"),
                        role: row.get("role"),
                        password_salt: row.try_get("password_salt").unwrap_or(None),
                        account_status: AccountStatus::from_str(row.get("account_status")),
                        email_verified: row.get("email_verified"),
                        phone_number: row.try_get("phone_number").unwrap_or(None),
                        phone_verified: row.get("phone_verified"),
                        display_name: row.try_get("display_name").unwrap_or(None),
                        first_name: row.try_get("first_name").unwrap_or(None),
                        last_name: row.try_get("last_name").unwrap_or(None),
                    }
                })
                .collect();
            Ok(users)
        }

        // Get full user by ID (returns FULL User struct with all fields)
        pub async fn get_user(id: i64, pool: &sqlx::PgPool) -> Result<User, ServerFnError> {
            let row = sqlx::query("SELECT id, username, email, password_hash, role, password_salt, account_status::text, email_verified, phone_number, phone_verified, display_name, first_name, last_name FROM users WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
                password_salt: row.try_get("password_salt").unwrap_or(None),
                account_status: AccountStatus::from_str(row.get("account_status")),
                email_verified: row.get("email_verified"),
                phone_number: row.try_get("phone_number").unwrap_or(None),
                phone_verified: row.get("phone_verified"),
                display_name: row.try_get("display_name").unwrap_or(None),
                first_name: row.try_get("first_name").unwrap_or(None),
                last_name: row.try_get("last_name").unwrap_or(None),
            };
            Ok(user)
        }

        //Get user settings
        pub async fn get_user_settings(
            pool: &Pool<Postgres>,
            user_id: i64
        ) -> Result<UserSettings, ServerFnError> {
            let row = sqlx::query("SELECT settings FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let settings_json: serde_json::Value = row.get("settings");
            let settings: UserSettings = serde_json::from_value(settings_json).unwrap_or_else(|_| UserSettings::default());

            Ok(settings)
        }

        // Get user by username for AUTHENTICATION - returns FULL User (needs password_hash for verification)
        pub async fn get_user_by_username(
            pool: &Pool<Postgres>,
            username: &str,
        ) -> Result<Option<User>, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, username, email, password_hash, role, password_salt, account_status::text, email_verified, phone_number, phone_verified, display_name, first_name, last_name
                 FROM users
                 WHERE username = $1"
            )
                .bind(&username)
                .fetch_optional(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            match row {
                Some(row) => {
                    let user = User {
                        id: row.get("id"),
                        username: row.get("username"),
                        email: row.get("email"),
                        password_hash: row.get("password_hash"),
                        role: row.get("role"),
                        password_salt: row.try_get("password_salt").unwrap_or(None),
                        account_status: AccountStatus::from_str(row.get("account_status")),
                        email_verified: row.get("email_verified"),
                        phone_number: row.try_get("phone_number").unwrap_or(None),
                        phone_verified: row.get("phone_verified"),
                        display_name: row.try_get("display_name").unwrap_or(None),
                        first_name: row.try_get("first_name").unwrap_or(None),
                        last_name: row.try_get("last_name").unwrap_or(None),
                    };
                    Ok(Some(user))
                },
                None => Ok(None),
            }
        }

        // Get user by email for PASSWORD RESET - returns FULL User
        pub async fn get_user_by_email(
            pool: &Pool<Postgres>,
            email: &str,
        ) -> Result<Option<User>, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, username, email, password_hash, role, password_salt, account_status::text, email_verified, phone_number, phone_verified, display_name, first_name, last_name
                 FROM users
                 WHERE email = $1"
            )
                .bind(&email)
                .fetch_optional(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            match row {
                Some(row) => {
                    let user = User {
                        id: row.get("id"),
                        username: row.get("username"),
                        email: row.get("email"),
                        password_hash: row.get("password_hash"),
                        role: row.get("role"),
                        password_salt: row.try_get("password_salt").unwrap_or(None),
                        account_status: AccountStatus::from_str(row.get("account_status")),
                        email_verified: row.get("email_verified"),
                        phone_number: row.try_get("phone_number").unwrap_or(None),
                        phone_verified: row.get("phone_verified"),
                        display_name: row.try_get("display_name").unwrap_or(None),
                        first_name: row.try_get("first_name").unwrap_or(None),
                        last_name: row.try_get("last_name").unwrap_or(None),
                    };
                    Ok(Some(user))
                },
                None => Ok(None),
            }
        }

        // Get user by ID for general purposes - returns lightweight SessionUser
        pub async fn get_user_by_id(pool: &Pool<Postgres>, id: i64) -> Result<Option<SessionUser>, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, username, email, role, display_name, first_name, last_name
                FROM users
                WHERE id = $1"
            )
                .bind(&id)
                .fetch_optional(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            match row {
                Some(row) => {
                    let user = SessionUser {
                        id: row.get("id"),
                        username: row.get("username"),
                        email: row.get("email"),
                        role: row.get("role"),
                        display_name: row.try_get("display_name").unwrap_or(None),
                        first_name: row.try_get("first_name").unwrap_or(None),
                        last_name: row.try_get("last_name").unwrap_or(None),
                    };
                    Ok(Some(user))
                },
                None => Ok(None),
            }
        }

        //Update user settings
        pub async fn update_user_settings(pool: &Pool<Postgres>, user_id: i64, settings_update: UserSettingsUpdate) -> Result<UserSettings, ServerFnError> {
            let current_settings = get_user_settings(pool, user_id).await?;

            //apply settings
            let mut new_settings = current_settings;

            if let Some(ui_update) = settings_update.ui {
                if let Some(dark_mode) = ui_update.dark_mode {
                    new_settings.ui.dark_mode = dark_mode;
                }
                if let Some(pinned_sidebar) = ui_update.pinned_sidebar {
                    new_settings.ui.pinned_sidebar = pinned_sidebar;
                }
            }

            let settings_json = serde_json::to_value(&new_settings)
                .map_err(|e| ServerFnError::new(format!("Failed to serialize settings: {}", e)))?;

            sqlx::query("UPDATE users SET settings = $1 WHERE id = $2")
                .bind(&settings_json)
                .bind(user_id)
                .execute(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to update user settings: {}", e)))?;

            Ok(new_settings)
        }

        //Reset user settings to default
        pub async fn reset_user_settings(pool: &Pool<Postgres>, user_id: i64) -> Result<UserSettings, ServerFnError> {
            let default_settings = UserSettings::default();
            let settings_json = serde_json::to_value(&default_settings)
                .map_err(|e| ServerFnError::new(format!("Failed to serialize default settings: {}", e)))?;

            sqlx::query("UPDATE users SET settings = $1 WHERE id = $2")
                .bind(&settings_json)
                .bind(user_id)
                .execute(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to reset user settings: {}", e)))?;

            Ok(default_settings)
        }

        // Session Management
        pub async fn create_session(pool: &Pool<Postgres>, user_id: i64) -> Result<String, ServerFnError> {
            let session_token = uuid::Uuid::new_v4().to_string();

            sqlx::query("INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, NOW() + INTERVAL '7 days')")
                .bind(user_id)
                .bind(&session_token)
                .execute(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Error creating session: {}", e)))?;

            Ok(session_token)
        }

        // Validate session and return SessionUser (MAIN SESSION VALIDATION FUNCTION)
        // Returns SessionUser - safe for session storage and client use
        pub async fn validate_session(
            pool: &Pool<Postgres>,
            session_token: &str,
        ) -> Result<Option<SessionUser>, ServerFnError> {
            let row = sqlx::query(
                "SELECT u.id, u.username, u.email, u.role, u.display_name, u.first_name, u.last_name
                 FROM users u
                 JOIN sessions s ON u.id = s.user_id
                 WHERE s.token = $1 AND s.expires_at > NOW()"
            )
            .bind(&session_token)
            .fetch_optional(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Error validating session: {}", e)))?;

            match row {
                Some(row) => {
                    let user = SessionUser {
                        id: row.get("id"),
                        username: row.get("username"),
                        email: row.get("email"),
                        role: row.get("role"),
                        display_name: row.try_get("display_name").unwrap_or(None),
                        first_name: row.try_get("first_name").unwrap_or(None),
                        last_name: row.try_get("last_name").unwrap_or(None),
                    };
                    Ok(Some(user))
                },
                None => Ok(None),
            }
        }

        // Delete a session (logout)
        pub async fn delete_session(pool: &Pool<Postgres>, token: &str) -> Result<(), ServerFnError> {
            sqlx::query("DELETE FROM sessions WHERE token = $1")
                .bind(token)
                .execute(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Error deleting session: {}", e)))?;

            Ok(())
        }

        // Clean up expired sessions (call this periodically)
        pub async fn cleanup_expired_sessions(pool: &Pool<Postgres>) -> Result<u64, ServerFnError> {
            let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= NOW()")
                .execute(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Error cleaning up sessions: {}", e)))?;

            Ok(result.rows_affected())
        }

        // Password Reset Functions
        pub async fn set_password_reset_token(
            pool: &Pool<Postgres>,
            user_id: i64,
            token: &str,
            expires: DateTime<Utc>
        ) -> Result<(), ServerFnError> {
            sqlx::query(
                "UPDATE users
                 SET password_reset_token = $1, password_reset_expires = $2 
                 WHERE id = $3"
            )
            .bind(token)
            .bind(expires)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to set reset token: {}", e)))?;

            Ok(())
        }

        pub async fn validate_password_reset_token(
            pool: &Pool<Postgres>,
            token: &str
        ) -> Result<bool, ServerFnError> {
            let result = sqlx::query(
                "SELECT COUNT(*) as count
                 FROM users 
                 WHERE password_reset_token = $1 
                 AND password_reset_expires > NOW()"
            )
            .bind(token)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Error validating token: {}", e)))?;

            let count: i64 = result.get("count");
            Ok(count > 0)
        }

        pub async fn get_user_by_reset_token(
            pool: &Pool<Postgres>,
            token: &str
        ) -> Result<Option<User>, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, username, email, password_hash, role, password_salt, account_status::text, email_verified, phone_number, phone_verified, display_name, first_name, last_name
                 FROM users
                 WHERE password_reset_token = $1
                 AND password_reset_expires > NOW()"
            )
            .bind(token)
            .fetch_optional(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Error getting user by token: {}", e)))?;

            match row {
                Some(row) => {
                    let user = User {
                        id: row.get("id"),
                        username: row.get("username"),
                        email: row.get("email"),
                        password_hash: row.get("password_hash"),
                        role: row.get("role"),
                        password_salt: row.try_get("password_salt").unwrap_or(None),
                        account_status: AccountStatus::from_str(row.get("account_status")),
                        email_verified: row.get("email_verified"),
                        phone_number: row.try_get("phone_number").unwrap_or(None),
                        phone_verified: row.get("phone_verified"),
                        display_name: row.try_get("display_name").unwrap_or(None),
                        first_name: row.try_get("first_name").unwrap_or(None),
                        last_name: row.try_get("last_name").unwrap_or(None),
                    };
                    Ok(Some(user))
                },
                None => Ok(None),
            }
        }

        pub async fn update_password_and_clear_token(
            pool: &Pool<Postgres>,
            user_id: i64,
            password_hash: &str
        ) -> Result<(), ServerFnError> {
            sqlx::query(
                "UPDATE users
                 SET password_hash = $1, password_reset_token = NULL, password_reset_expires = NULL 
                 WHERE id = $2"
            )
            .bind(password_hash)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to update password: {}", e)))?;

            Ok(())
        }

        // Admin Functions
        pub async fn update_permissions(user_id: i64, role: UserRole, pool: &sqlx::PgPool) -> Result<User, ServerFnError> {
            let row = sqlx::query("UPDATE users SET role = $1 WHERE id = $2 RETURNING id, username, email, password_hash, role, password_salt, account_status::text, email_verified, phone_number, phone_verified, display_name, first_name, last_name")
                .bind(&role)
                .bind(user_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to update user permissions: {}", e)))?;

            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
                password_salt: row.try_get("password_salt").unwrap_or(None),
                account_status: AccountStatus::from_str(row.get("account_status")),
                email_verified: row.get("email_verified"),
                phone_number: row.try_get("phone_number").unwrap_or(None),
                phone_verified: row.get("phone_verified"),
                display_name: row.try_get("display_name").unwrap_or(None),
                first_name: row.try_get("first_name").unwrap_or(None),
                last_name: row.try_get("last_name").unwrap_or(None),
            };
            Ok(user)
        }

        pub async fn update_user_data(new_user_data: User, pool: &sqlx::PgPool) -> Result<User, ServerFnError> {
            let row = sqlx::query("UPDATE users SET username = $1, phone_number = $2, first_name = $3, last_name = $4 WHERE id = $5 RETURNING id, username, email, password_hash, role, password_salt, account_status::text, email_verified, phone_number, phone_verified, display_name, first_name, last_name")
                .bind(new_user_data.username)
                .bind(new_user_data.phone_number)
                .bind(new_user_data.first_name)
                .bind(new_user_data.last_name)
                .bind(new_user_data.id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to update user: {}", e)))?;

            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
                password_salt: row.try_get("password_salt").unwrap_or(None),
                account_status: AccountStatus::from_str(row.get("account_status")),
                email_verified: row.get("email_verified"),
                phone_number: row.try_get("phone_number").unwrap_or(None),
                phone_verified: row.get("phone_verified"),
                display_name: row.try_get("display_name").unwrap_or(None),
                first_name: row.try_get("first_name").unwrap_or(None),
                last_name: row.try_get("last_name").unwrap_or(None),
            };
            Ok(user)
        }
    }
}
