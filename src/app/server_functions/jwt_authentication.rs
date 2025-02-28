/*use crate::app::models::user::User;
use crate::app::models::user::{AuthResponse, Claims, LoginCredentials, UserPublic};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use leptos::*;
use time::OffsetDateTime;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use {
    actix_web::HttpRequest,
    sqlx::{postgres::PgPoolOptions, PgPool, Row},
};

fn get_jwt_secret() -> Result<String, ServerFnError> {
    std::env::var("JWT_SECRET").map_err(|_| ServerFnError::new("JWT_SECRET must be set"))
}

#[server(Login, "/api/auth")]
pub async fn login(credentials: LoginCredentials) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let row = sqlx::query(
            "SELECT id, username, password_hash, email, created_at FROM users WHERE username = $1",
        )
        .bind(credentials.username)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

        let user = User {
            id: row.get("id"),
            username: row.get("username"),
            password_hash: row.get("password_hash"),
            email: row.get("email"),
            created_at: row.get("created_at"),
        };

        if !verify_password(&credentials.password, &user.password_hash)? {
            return Err(ServerFnError::new("Invalid credentials"));
        }

        let token = generate_token(&user.id.to_string())?;

        Ok(AuthResponse {
            token,
            user: UserPublic {
                id: user.id.to_string(),
                username: user.username,
            },
        })
    }
}

#[server(Register, "/api/auth")]
pub async fn register(credentials: LoginCredentials) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let row = sqlx::query("SELECT id FROM users WHERE username = $1")
            .bind(&credentials.username)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

        let existing_user: Option<String> = Some(row.get("id"));

        if existing_user.is_some() {
            return Err(ServerFnError::new("Username already taken"));
        }

        let password_hash = hash_password(&credentials.password)?;

        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, username, password_hash, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(user_id)
        .bind(&credentials.username)
        .bind(password_hash)
        .bind(time::OffsetDateTime::now_utc())
        .execute(pool.get_ref())
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create user: {}", e)))?;

        Ok(())
    }
}

#[server(GetCurrentUser, "/api/auth")]
pub async fn get_current_user() -> Result<Option<UserPublic>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let req = extract::<HttpRequest>().await?;

        let auth_header = req.headers().get("Authorization");
        if let Some(auth_value) = auth_header {
            if let Ok(auth_str) = auth_value.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];

                    //validate token
                    if let Ok(claims) = validate_token(token) {
                        let pool = extract::<web::Data<PgPool>>().await.map_err(|e| {
                            ServerFnError::new(format!("Failed to extract pool: {}", e))
                        })?;
                        let user_id = Uuid::parse_str(&claims.subject)
                            .map_err(|e| ServerFnError::new("Invalid user ID"))?;

                        let row = sqlx::query("SELECT id, username, password_hash, email, created_at FROM users WHERE id = $1")
                            .bind(user_id)
                            .fetch_one(pool.get_ref())
                            .await
                            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

                        let user = User {
                            id: row.get("id"),
                            username: row.get("username"),
                            password_hash: row.get("password_hash"),
                            email: row.get("email"),
                            created_at: row.get("created_at"),
                        };

                        return Ok(Some(UserPublic {
                            id: user.id.to_string(),
                            username: user.username,
                        }));
                    }
                }
            }
        }
        //in the even that no valid token
        Ok(None)
    }
}

#[server(Logout, "/api/auth")]
pub async fn logout() -> Result<(), ServerFnError> {
    Ok(())
}

fn generate_token(user_id: &str) -> Result<String, ServerFnError> {
    let jwt_secret = get_jwt_secret()?;
    let now = OffsetDateTime::now_utc().unix_timestamp() as usize;
    let expiration = now + 24 * 3600; // 24 hours from now

    let claims = Claims {
        subject: user_id.to_string(),
        expiration,
        issued_at: now,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| ServerFnError::new(format!("Error creating token: {}", e)))?;

    Ok(token)
}

fn validate_token(token: &str) -> Result<Claims, ServerFnError> {
    let jwt_secret = get_jwt_secret()?;
    let validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|e| ServerFnError::new(format!("Error validating token: {}", e)))?;

    Ok(token_data.claims)
}

fn hash_password(password: &str) -> Result<String, ServerFnError> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ServerFnError::new(format!("Error hashing password: {}", e)))?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, ServerFnError> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| ServerFnError::new(format!("Error parsing hash: {}", e)))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}*/
