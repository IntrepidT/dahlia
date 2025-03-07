use crate::app::db::user_database;
use crate::app::models::user::User;
#[cfg(feature = "ssr")]
use actix_web::{cookie::Cookie, http::header, HttpRequest};
use leptos::*;
#[cfg(feature = "ssr")]
use leptos_actix::{extract, ResponseOptions};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::PgPool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<User>,
}

#[server(Login, "/api")]
pub async fn login(username: String, password: String) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        log::info!("Login attempt for user: {}", username);

        // Get the database connection pool
        use actix_web::web;
        use leptos_actix::extract;
        let pool = match extract::<web::Data<PgPool>>().await {
            Ok(pool) => pool,
            Err(e) => {
                let err_msg = format!("Failed to extract pool: {}", e);
                log::error!("{}", err_msg);
                return Err(ServerFnError::new(err_msg));
            }
        };

        // Check if the user exists
        log::info!("Looking up user in database: {}", username);
        let user_result = user_database::get_user_by_username(&pool, &username).await;

        match user_result {
            Ok(Some(user)) => {
                log::info!("User found, verifying password");
                // Verify the password
                if user_database::verify_password(&password, &user.password_hash) {
                    log::info!("Password verified for user: {}", username);
                    // Create a session
                    match user_database::create_session(&pool, user.id).await {
                        Ok(session_token) => {
                            log::info!("Session created for user: {}", username);
                            // Set the session cookie
                            let response = expect_context::<ResponseOptions>();
                            response.insert_header(
                                header::SET_COOKIE,
                                header::HeaderValue::from_str(&format!(
                                    "session={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=604800",
                                    session_token
                                ))
                                .expect("Failed to create header value"),
                            );

                            // Return success
                            let auth_response = AuthResponse {
                                success: true,
                                message: "Login successful".to_string(),
                                user: Some(user),
                            };
                            log::info!("Login successful for user: {}", username);
                            Ok(auth_response)
                        }
                        Err(e) => {
                            log::error!("Failed to create session: {:?}", e);
                            Ok(AuthResponse {
                                success: false,
                                message: "Failed to create session".to_string(),
                                user: None,
                            })
                        }
                    }
                } else {
                    log::info!("Invalid password for user: {}", username);
                    Ok(AuthResponse {
                        success: false,
                        message: "Invalid password".to_string(),
                        user: None,
                    })
                }
            }
            Ok(None) => {
                log::info!("User not found: {}", username);
                Ok(AuthResponse {
                    success: false,
                    message: "User not found".to_string(),
                    user: None,
                })
            }
            Err(e) => {
                log::error!("Database error when looking up user: {:?}", e);
                Ok(AuthResponse {
                    success: false,
                    message: "Database error".to_string(),
                    user: None,
                })
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        log::warn!("Login function called without SSR feature enabled");
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server(Logout, "/api")]
pub async fn logout() -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        // Get the database connection pool
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Get the session token from the cookie
        let req = expect_context::<HttpRequest>();
        let cookies = req.cookies().unwrap();

        // Find the session cookie
        let session_token_opt = cookies
            .iter()
            .find(|c| c.name() == "session")
            .map(|c| c.value().to_string());

        if let Some(session_token) = session_token_opt {
            // Delete the session
            if let Err(_) = user_database::delete_session(&pool, &session_token).await {
                return Ok(AuthResponse {
                    success: false,
                    message: "Failed to delete session".to_string(),
                    user: None,
                });
            }
        }

        // Clear the session cookie
        let response = expect_context::<ResponseOptions>();
        response.insert_header(
            header::SET_COOKIE,
            header::HeaderValue::from_str("session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0")
                .expect("Failed to create header value"),
        );

        Ok(AuthResponse {
            success: true,
            message: "Logout successful".to_string(),
            user: None,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server(GetCurrentUser, "/api")]
pub async fn get_current_user() -> Result<Option<User>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        // Get the database connection pool
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Get the session token from the cookie
        let req = expect_context::<HttpRequest>();
        let cookies = req.cookies().unwrap();
        log::info!("Attempting to find session cookie");
        // Find the session cookie
        let session_token_opt = cookies
            .iter()
            .find(|c| c.name() == "session")
            .map(|c| c.value().to_string());

        if let Some(session_token) = session_token_opt {
            // Get the user from the session
            log::info!("Attempting to match user with database via session");
            match user_database::get_user_by_session(&pool, &session_token).await {
                Ok(Some(user)) => {
                    return Ok(Some(user));
                }
                _ => {}
            }
        }

        Ok(None)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server(Register, "/api")]
pub async fn register(
    username: String,
    email: String,
    password: String,
) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        // Get the database connection pool
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to determine whether username exists");
        // Check if the username already exists
        if let Ok(Some(_)) = user_database::get_user_by_username(&pool, &username).await {
            return Ok(AuthResponse {
                success: false,
                message: "Username already exists".to_string(),
                user: None,
            });
        }

        log::info!("Attempting to create new user");
        // Create the user
        match user_database::create_user(&pool, username, email, password, "user".to_string()).await
        {
            Ok(user) => {
                // Create a session
                match user_database::create_session(&pool, user.id).await {
                    Ok(session_token) => {
                        // Set the session cookie
                        let response = expect_context::<ResponseOptions>();
                        response.insert_header(
                            header::SET_COOKIE,
                            header::HeaderValue::from_str(&format!(
                                "session={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=604800",
                                session_token
                            ))
                            .expect("Failed to create header value"),
                        );

                        // Return success
                        Ok(AuthResponse {
                            success: true,
                            message: "Registration successful".to_string(),
                            user: Some(user),
                        })
                    }
                    Err(_) => Ok(AuthResponse {
                        success: false,
                        message: "Failed to create session".to_string(),
                        user: None,
                    }),
                }
            }
            Err(_) => Ok(AuthResponse {
                success: false,
                message: "Failed to create user".to_string(),
                user: None,
            }),
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}
