use crate::app::db::user_database;
use crate::app::models::user::{SessionUser, UserRole};
#[cfg(feature = "ssr")]
use actix_web::{cookie::Cookie, http::header, HttpRequest, HttpResponse};
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
    pub user: Option<SessionUser>,
}

#[server(Login, "/api")]
pub async fn login(username: String, password: String) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        log::info!("Login attempt for user: {}", username);

        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Looking up user in database: {}", username);
        let user_result = user_database::get_user_by_username(&pool, &username).await;

        match user_result {
            Ok(Some(user)) => {
                log::info!("User found, verifying password");
                if user_database::verify_password(&password, &user.password_hash) {
                    log::info!("Password verified for user: {}", username);

                    match user_database::create_session(&pool, user.id).await {
                        Ok(session_token) => {
                            log::info!("Session created for user: {}", username);

                            // Set secure session cookie
                            let response = expect_context::<ResponseOptions>();
                            let cookie_value =
                                format!(
                                "session={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=604800{}",
                                session_token,
                                if cfg!(debug_assertions) { "" } else { "; Secure" }
                            );

                            response.insert_header(
                                header::SET_COOKIE,
                                header::HeaderValue::from_str(&cookie_value)
                                    .expect("Failed to create header value"),
                            );

                            Ok(AuthResponse {
                                success: true,
                                message: "Login successful".to_string(),
                                user: Some(user.to_session_user()), // Convert to SessionUser
                            })
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
                        message: "Invalid credentials".to_string(),
                        user: None,
                    })
                }
            }
            Ok(None) => {
                log::info!("User not found: {}", username);
                Ok(AuthResponse {
                    success: false,
                    message: "Invalid credentials".to_string(),
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
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server(Logout, "/api")]
pub async fn logout() -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let req = extract::<HttpRequest>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract request: {}", e)))?;

        let cookies = match req.cookies() {
            Ok(cookies) => cookies,
            Err(e) => {
                log::error!("Failed to extract cookies: {:?}", e);
                return Ok(AuthResponse {
                    success: false,
                    message: "Failed to parse cookies".to_string(),
                    user: None,
                });
            }
        };

        // Find and invalidate session
        if let Some(session_cookie) = cookies.iter().find(|c| c.name() == "session") {
            let session_token = session_cookie.value();
            if let Err(e) = user_database::delete_session(&pool, session_token).await {
                log::error!("Failed to delete session from database: {:?}", e);
            }
        }

        // Clear the session cookie
        let response = expect_context::<ResponseOptions>();
        let clear_cookie = format!(
            "session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{}",
            if cfg!(debug_assertions) {
                ""
            } else {
                "; Secure"
            }
        );

        response.insert_header(
            header::SET_COOKIE,
            header::HeaderValue::from_str(&clear_cookie).expect("Failed to create header value"),
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
pub async fn get_current_user() -> Result<Option<SessionUser>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let req = extract::<HttpRequest>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract request: {}", e)))?;

        let cookies = match req.cookies() {
            Ok(cookies) => cookies,
            Err(e) => {
                log::error!("Failed to extract cookies: {:?}", e);
                return Ok(None);
            }
        };

        if let Some(session_cookie) = cookies.iter().find(|c| c.name() == "session") {
            let session_token = session_cookie.value();

            match user_database::validate_session(&pool, session_token).await {
                Ok(Some(user)) => {
                    log::debug!("Valid session found for user: {}", user.username);
                    return Ok(Some(user)); // Already returns SessionUser
                }
                Ok(None) => {
                    log::debug!("Invalid or expired session");
                }
                Err(e) => {
                    log::error!("Error validating session: {:?}", e);
                }
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
        // Input validation
        if username.trim().is_empty() || email.trim().is_empty() || password.len() < 8 {
            return Ok(AuthResponse {
                success: false,
                message: "Invalid input: username and email cannot be empty, password must be at least 8 characters".to_string(),
                user: None,
            });
        }

        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Registration attempt for username: {}", username);

        // Check for existing username
        if let Ok(Some(_)) = user_database::get_user_by_username(&pool, &username).await {
            return Ok(AuthResponse {
                success: false,
                message: "Username already exists".to_string(),
                user: None,
            });
        }

        // Check for existing email
        if let Ok(Some(_)) = user_database::get_user_by_email(&pool, &email).await {
            return Ok(AuthResponse {
                success: false,
                message: "Email already exists".to_string(),
                user: None,
            });
        }

        // Create user
        match user_database::create_user(&pool, username.clone(), email, password, UserRole::Guest)
            .await
        {
            Ok(user) => {
                log::info!("User created successfully: {}", username);

                // Create session
                match user_database::create_session(&pool, user.id).await {
                    Ok(session_token) => {
                        // Set session cookie
                        let response = expect_context::<ResponseOptions>();
                        let cookie_value = format!(
                            "session={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=604800{}",
                            session_token,
                            if cfg!(debug_assertions) {
                                ""
                            } else {
                                "; Secure"
                            }
                        );

                        response.insert_header(
                            header::SET_COOKIE,
                            header::HeaderValue::from_str(&cookie_value)
                                .expect("Failed to create header value"),
                        );

                        Ok(AuthResponse {
                            success: true,
                            message: "Registration successful".to_string(),
                            user: Some(user.to_session_user()), // Convert to SessionUser
                        })
                    }
                    Err(e) => {
                        log::error!("Failed to create session for new user: {:?}", e);
                        Ok(AuthResponse {
                            success: false,
                            message: "Failed to create session".to_string(),
                            user: None,
                        })
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create user: {:?}", e);
                Ok(AuthResponse {
                    success: false,
                    message: "Failed to create user".to_string(),
                    user: None,
                })
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Keep password reset functions as they are - they're well implemented
#[server(RequestPasswordReset, "/api")]
pub async fn request_password_reset(email: String) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::services::email_service;
        use actix_web::web;
        use chrono::{Duration, Utc};
        use leptos_actix::extract;
        use rand::{distributions::Alphanumeric, Rng};

        log::info!("Password reset requested for email: {}", email);

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let user_result = user_database::get_user_by_email(&pool, &email).await;

        match user_result {
            Ok(Some(user)) => {
                let token: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(64)
                    .map(char::from)
                    .collect();

                let expires = Utc::now() + Duration::hours(24);

                match user_database::set_password_reset_token(&pool, user.id, &token, expires).await
                {
                    Ok(_) => {
                        if let Err(e) = email_service::send_reset_email(&email, &token).await {
                            log::error!("Failed to send password reset email: {}", e);
                        }

                        Ok(AuthResponse {
                            success: true,
                            message: "Password reset instructions sent to your email".to_string(),
                            user: None,
                        })
                    }
                    Err(e) => {
                        log::error!("Failed to set password reset token: {:?}", e);
                        Ok(AuthResponse {
                            success: false,
                            message: "Failed to initiate password reset".to_string(),
                            user: None,
                        })
                    }
                }
            }
            Ok(None) => {
                log::info!("Password reset requested for non-existent email: {}", email);
                Ok(AuthResponse {
                    success: true,
                    message:
                        "If this email is registered, password reset instructions have been sent"
                            .to_string(),
                    user: None,
                })
            }
            Err(e) => {
                log::error!("Database error looking up user by email: {:?}", e);
                Ok(AuthResponse {
                    success: false,
                    message: "An error occurred processing your request".to_string(),
                    user: None,
                })
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server(ValidateResetToken, "/api")]
pub async fn validate_reset_token(token: String) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        match user_database::validate_password_reset_token(&pool, &token).await {
            Ok(valid) => Ok(valid),
            Err(e) => {
                log::error!("Error validating reset token: {:?}", e);
                Err(ServerFnError::new("Database error".to_string()))
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server(ResetPassword, "/api")]
pub async fn reset_password(
    token: String,
    new_password: String,
) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        if new_password.len() < 8 {
            return Ok(AuthResponse {
                success: false,
                message: "Password must be at least 8 characters long".to_string(),
                user: None,
            });
        }

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        if let Ok(true) = user_database::validate_password_reset_token(&pool, &token).await {
            match user_database::get_user_by_reset_token(&pool, &token).await {
                Ok(Some(user)) => {
                    let password_hash = user_database::hash_password(&new_password)?;

                    match user_database::update_password_and_clear_token(
                        &pool,
                        user.id,
                        &password_hash,
                    )
                    .await
                    {
                        Ok(_) => Ok(AuthResponse {
                            success: true,
                            message: "Password successfully reset".to_string(),
                            user: None,
                        }),
                        Err(e) => {
                            log::error!("Failed to update password: {:?}", e);
                            Ok(AuthResponse {
                                success: false,
                                message: "Failed to reset password".to_string(),
                                user: None,
                            })
                        }
                    }
                }
                Ok(None) => Ok(AuthResponse {
                    success: false,
                    message: "Invalid reset token".to_string(),
                    user: None,
                }),
                Err(e) => {
                    log::error!("Database error looking up user by reset token: {:?}", e);
                    Ok(AuthResponse {
                        success: false,
                        message: "An error occurred processing your request".to_string(),
                        user: None,
                    })
                }
            }
        } else {
            Ok(AuthResponse {
                success: false,
                message: "Invalid or expired reset token".to_string(),
                user: None,
            })
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}
