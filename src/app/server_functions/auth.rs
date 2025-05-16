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

#[server(RequestPasswordReset, "/api")]
pub async fn request_password_reset(email: String) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use rand::{distributions::Alphanumeric, Rng};
        use chrono::{Utc, Duration};
        use crate::app::services::email_service;
        
        log::info!("Password reset requested for email: {}", email);
        
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;
        
        // Check if the user exists
        let user_result = user_database::get_user_by_email(&pool, &email).await;
        
        match user_result {
            Ok(Some(user)) => {
                // Generate a random token
                let token: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(64)
                    .map(char::from)
                    .collect();
                
                // Set expiration time (24 hours from now)
                let expires = Utc::now() + Duration::hours(24);
                
                // Update the user in the database with the reset token and expiration
                match user_database::set_password_reset_token(&pool, user.id, &token, expires).await {
                    Ok(_) => {
                        // Send an email with the password reset link
                        if let Err(e) = email_service::send_reset_email(&email, &token).await {
                            log::error!("Failed to send password reset email: {}", e);
                            // Continue the process even if email sending fails
                            // We'll still return success to the user
                        }
                        
                        Ok(AuthResponse {
                            success: true,
                            message: "Password reset instructions sent to your email".to_string(),
                            user: None,
                        })
                    },
                    Err(e) => {
                        log::error!("Failed to set password reset token: {:?}", e);
                        Ok(AuthResponse {
                            success: false,
                            message: "Failed to initiate password reset".to_string(),
                            user: None,
                        })
                    }
                }
            },
            Ok(None) => {
                // Don't reveal that the email doesn't exist for security reasons
                // Instead, pretend we sent the email anyway
                log::info!("Password reset requested for non-existent email: {}", email);
                Ok(AuthResponse {
                    success: true,
                    message: "If this email is registered, password reset instructions have been sent".to_string(),
                    user: None,
                })
            },
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
        
        // Check if the token exists and is not expired
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
pub async fn reset_password(token: String, new_password: String) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;
        
        // Validate token again before proceeding
        if let Ok(true) = user_database::validate_password_reset_token(&pool, &token).await {
            // Get the user associated with this token
            match user_database::get_user_by_reset_token(&pool, &token).await {
                Ok(Some(user)) => {
                    // Hash the new password
                    let password_hash = user_database::hash_password(&new_password)?;
                    
                    // Update the user's password and clear the reset token
                    match user_database::update_password_and_clear_token(&pool, user.id, &password_hash).await {
                        Ok(_) => {
                            Ok(AuthResponse {
                                success: true,
                                message: "Password successfully reset".to_string(),
                                user: None,
                            })
                        },
                        Err(e) => {
                            log::error!("Failed to update password: {:?}", e);
                            Ok(AuthResponse {
                                success: false,
                                message: "Failed to reset password".to_string(),
                                user: None,
                            })
                        }
                    }
                },
                Ok(None) => {
                    Ok(AuthResponse {
                        success: false,
                        message: "Invalid reset token".to_string(),
                        user: None,
                    })
                },
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
