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

// Test cases for the module
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Helper function to check if we're in a testing environment with proper config
    fn has_email_config() -> bool {
        env::var("SMTP_USERNAME").is_ok()
            && env::var("SMTP_PASSWORD").is_ok()
            && env::var("TEST_EMAIL").is_ok()
    }

    // Test that validates email configuration without sending
    #[test]
    fn test_email_config_validation() {
        // Load .env for testing
        dotenv::dotenv().ok();

        if !has_email_config() {
            println!("⚠️  Skipping email config test - missing environment variables");
            println!("   Set SMTP_USERNAME, SMTP_PASSWORD, and TEST_EMAIL to run email tests");
            return;
        }

        let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".to_string());
        let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME should be set");
        let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD should be set");
        let from_email = env::var("FROM_EMAIL").unwrap_or_else(|| smtp_username.clone());

        // Validate configuration
        assert_eq!(
            smtp_server, "smtp.gmail.com",
            "Should use Gmail SMTP server"
        );
        assert!(
            smtp_username.ends_with("@teapottesting.com"),
            "Username should be from teapottesting.com domain"
        );
        assert!(
            from_email.ends_with("@teapottesting.com"),
            "From email should be from teapottesting.com domain"
        );

        // Validate password format (16 chars with or without spaces)
        let password_clean = smtp_password.replace(" ", "");
        assert_eq!(
            password_clean.len(),
            16,
            "SMTP password should be 16 characters (Google App Password)"
        );

        println!("✅ Email configuration validation passed");
        println!("   SMTP Server: {}", smtp_server);
        println!("   Username: {}", smtp_username);
        println!("   From Email: {}", from_email);
        println!("   Password Length: {} chars", password_clean.len());
    }

    // Test that actually sends an email (requires network and valid config)
    #[tokio::test]
    async fn test_send_reset_email() {
        // Load .env for testing
        dotenv::dotenv().ok();

        if !has_email_config() {
            println!("⚠️  Skipping email send test - missing environment variables");
            println!("   Set SMTP_USERNAME, SMTP_PASSWORD, and TEST_EMAIL to run email tests");
            return;
        }

        let test_email =
            env::var("TEST_EMAIL").expect("TEST_EMAIL should be set for integration tests");

        // Import the email service
        #[cfg(feature = "ssr")]
        {
            use crate::app::services::email_service;

            println!("🧪 Testing email send to: {}", test_email);

            match email_service::send_reset_email(&test_email, "test-token-from-cargo-test").await {
                Ok(_) => {
                    println!("✅ Test email sent successfully!");
                    println!("   Check {} for the test email", test_email);
                    println!("   Subject: TeaPot Testing - Password Reset Request");
                }
                Err(e) => {
                    panic!("❌ Failed to send test email: {}", e);
                }
            }
        }

        #[cfg(not(feature = "ssr"))]
        {
            println!("⚠️  Email sending test skipped - not in SSR mode");
        }
    }

    // Test password reset token generation and validation
    #[tokio::test]
    async fn test_password_reset_flow() {
        dotenv::dotenv().ok();

        if !has_email_config() {
            println!("⚠️  Skipping password reset flow test - missing environment variables");
            return;
        }

        let test_email = env::var("TEST_EMAIL").expect("TEST_EMAIL should be set");

        #[cfg(feature = "ssr")]
        {
            println!("🧪 Testing full password reset flow for: {}", test_email);

            // Test the request_password_reset function
            match request_password_reset(test_email.clone()).await {
                Ok(response) => {
                    assert!(response.success, "Password reset request should succeed");
                    println!("✅ Password reset request successful");
                    println!("   Message: {}", response.message);
                    println!("   Check {} for the reset email", test_email);
                }
                Err(e) => {
                    panic!("❌ Password reset request failed: {:?}", e);
                }
            }
        }

        #[cfg(not(feature = "ssr"))]
        {
            println!("⚠️  Password reset flow test skipped - not in SSR mode");
        }
    }

    // Benchmark test to check email sending performance
    #[tokio::test]
    async fn test_email_performance() {
        dotenv::dotenv().ok();

        if !has_email_config() {
            println!("⚠️  Skipping email performance test - missing environment variables");
            return;
        }

        #[cfg(feature = "ssr")]
        {
            use crate::app::services::email_service;
            use std::time::Instant;

            let test_email = env::var("TEST_EMAIL").expect("TEST_EMAIL should be set");

            println!("⏱️  Testing email sending performance...");
            let start = Instant::now();

            match email_service::send_reset_email(&test_email, "performance-test-token").await {
                Ok(_) => {
                    let duration = start.elapsed();
                    println!("✅ Email sent in {:?}", duration);

                    // Email should send within reasonable time (30 seconds with our timeout)
                    assert!(
                        duration.as_secs() < 30,
                        "Email should send within 30 seconds"
                    );

                    if duration.as_secs() > 10 {
                        println!("⚠️  Email took longer than 10 seconds - consider checking network/SMTP config");
                    }
                }
                Err(e) => {
                    panic!("❌ Performance test failed: {}", e);
                }
            }
        }
    }
}
