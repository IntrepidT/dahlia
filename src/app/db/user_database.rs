cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {

        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
            Argon2,
        };
        use sqlx::{Pool, Postgres};
        use leptos::ServerFnError;
        use crate::app::models::user::User;
        use sqlx::Row;

        // Create a new user
        pub async fn create_user(
            pool: &Pool<Postgres>,
            username: String,
            email: String,
            password: String,
            role: String,
        ) -> Result<User, ServerFnError> {
            // Hash the password
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .unwrap()
                .to_string();

            let row = sqlx::query(
                "INSERT INTO users (username, email, password_hash, role)
                 VALUES ($1, $2, $3, $4)
                 RETURNING id, username, email, password_hash, role"
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
            };

            Ok(user)
        }

        // Get a user by username
        pub async fn get_user_by_username(
            pool: &Pool<Postgres>,
            username: &str,
        ) -> Result<Option<User>, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, username, email, password_hash, role
                 FROM users
                 WHERE username = $1"
            )
                .bind(&username)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
            };

            Ok(Some(user))
        }

        // Get a user by ID
        pub async fn get_user_by_id(pool: &Pool<Postgres>, id: i64) -> Result<Option<User>, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, username, email, password_hash, role
                FROM users
                WHERE id = $1"
            )
                .bind(&id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
            };

            Ok(Some(user))
        }

        // Verify a user's password
        pub fn verify_password(password: &str, password_hash: &str) -> bool {
            // Parse the password hash
            let parsed_hash = match PasswordHash::new(password_hash) {
                Ok(hash) => hash,
                Err(_) => return false,
            };

            // Verify the password
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        }

        // Create a session for a user
        pub async fn create_session(pool: &Pool<Postgres>, user_id: i64) -> Result<String, ServerFnError> {
            // Generate a random session token
            let session_token = uuid::Uuid::new_v4().to_string();

            // Insert the session into the database
            sqlx::query("INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, NOW() + INTERVAL '7 days')")
                .bind(user_id),
                .bind(session_token)
                .execute(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Error Inserting into sessions: {}", e)))?;

            Ok(session_token)
        }

        // Get a user by session token
        pub async fn get_user_by_session(
            pool: &Pool<Postgres>,
            token: &str,
        ) -> Result<Option<User>, ServerFnError> {
            let row = sqlx::query(
                "SELECT u.id, u.username, u.email, u.password_hash, u.role
                FROM users u
                JOIN sessions s ON u.id = s.user_id
                WHERE s.token = $1 AND s.expires_at > NOW()"
            )
            .bind(&token)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Error obtaining user via sessions: {}", e)))?;

            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
            };

            Ok(Some(user))
        }

        // Delete a session
        pub async fn delete_session(pool: &Pool<Postgres>, token: &str) -> Result<(), ServerFnError> {
            sqlx::query("DELETE FROM sessions WHERE token = $1")
                .bind(token)
                .execute(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Error deleting session: {}", e)))?;

            Ok(())
        }
    }
}
