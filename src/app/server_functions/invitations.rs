use leptos::prelude::*;
use crate::app::db::invitation_database;
use crate::app::models::invitation::{
    normalize_phone_number, CreateInvitationRequest, Invitation, InvitationInfo, VerificationType,
};
use crate::app::models::user::SessionUser;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use {actix_web::web, leptos_actix::extract, sqlx::PgPool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationResponse {
    pub success: bool,
    pub message: String,
    pub invitation: Option<Invitation>,
}

#[server]
pub async fn create_invitation(
    request: CreateInvitationRequest,
) -> Result<InvitationResponse, leptos::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::middleware::authentication;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| leptos::ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let req = extract::<actix_web::HttpRequest>()
            .await
            .map_err(|e| leptos::ServerFnError::new(format!("Failed to extract request: {}", e)))?;

        // Check if user is admin
        let current_user = authentication::get_current_user_from_request(&req);
        let user_id = match current_user {
            Some(user) if user.is_admin() => Some(user.id),
            Some(_) => {
                return Ok(InvitationResponse {
                    success: false,
                    message: "Admin access required".to_string(),
                    invitation: None,
                });
            }
            None => {
                return Ok(InvitationResponse {
                    success: false,
                    message: "Authentication required".to_string(),
                    invitation: None,
                });
            }
        };

        // Validate request
        if request.school_name.trim().is_empty() {
            return Ok(InvitationResponse {
                success: false,
                message: "School name is required".to_string(),
                invitation: None,
            });
        }

        if request.max_uses < 1 || request.max_uses > 1000 {
            return Ok(InvitationResponse {
                success: false,
                message: "Max uses must be between 1 and 1000".to_string(),
                invitation: None,
            });
        }

        if request.expires_in_days < 1 || request.expires_in_days > 365 {
            return Ok(InvitationResponse {
                success: false,
                message: "Expiration must be between 1 and 365 days".to_string(),
                invitation: None,
            });
        }

        match invitation_database::create_invitation(&pool, request, user_id).await {
            Ok(invitation) => Ok(InvitationResponse {
                success: true,
                message: "Invitation created successfully".to_string(),
                invitation: Some(invitation),
            }),
            Err(e) => {
                log::error!("Failed to create invitation: {:?}", e);
                Ok(InvitationResponse {
                    success: false,
                    message: "Failed to create invitation".to_string(),
                    invitation: None,
                })
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(leptos::ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server]
pub async fn validate_invitation(code: String) -> Result<Option<InvitationInfo>, leptos::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| leptos::ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        match invitation_database::get_invitation_by_code(&pool, &code).await {
            Ok(Some(invitation)) => {
                if invitation.can_be_used() {
                    Ok(Some(InvitationInfo {
                        code: invitation.code,
                        school_name: invitation.school_name,
                        role: invitation.role,
                        expires_at: invitation.expires_at,
                        uses_remaining: invitation.uses_remaining(),
                    }))
                } else {
                    Ok(None) // Invalid or expired
                }
            }
            Ok(None) => Ok(None),
            Err(e) => {
                log::error!("Failed to validate invitation: {:?}", e);
                Err(leptos::ServerFnError::new("Database error".to_string()))
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(leptos::ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub success: bool,
    pub message: String,
}

#[server]
pub async fn send_verification_code(
    user_id= i64,
    verification_type: String,
) -> Result<VerificationResponse, leptos::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| leptos::ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let v_type =
            VerificationType::from_str(&verification_type).map_err(|e| leptos::ServerFnError::new(e))?;

        // Rate limiting - max 3 codes per 15 minutes
        match invitation_database::count_recent_verification_codes(
            &pool,
            user_id,
            v_type.clone(),
            15,
        )
        .await
        {
            Ok(count) if count >= 3 => {
                return Ok(VerificationResponse {
                    success: false,
                    message: "Too many verification attempts. Please wait 15 minutes.".to_string(),
                });
            }
            Err(e) => {
                log::error!("Failed to check rate limit: {:?}", e);
            }
            _ => {} // Continue
        }

        // Get user info for sending codes
        let user = match crate::app::db::user_database::get_user_by_id(&pool, user_id).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Ok(VerificationResponse {
                    success: false,
                    message: "User not found".to_string(),
                });
            }
            Err(e) => {
                log::error!("Failed to get user: {:?}", e);
                return Ok(VerificationResponse {
                    success: false,
                    message: "Database error".to_string(),
                });
            }
        };

        // Create verification code
        match invitation_database::create_verification_code(&pool, user_id, v_type.clone()).await {
            Ok(verification_code) => {
                // Send the code
                match v_type {
                    VerificationType::Email => {
                        // Use your existing email service
                        match send_verification_email(&user.email, &verification_code.code).await {
                            Ok(_) => Ok(VerificationResponse {
                                success: true,
                                message: "Verification code sent to your email".to_string(),
                            }),
                            Err(e) => {
                                log::error!("Failed to send verification email: {}", e);
                                Ok(VerificationResponse {
                                    success: false,
                                    message: "Failed to send verification email".to_string(),
                                })
                            }
                        }
                    }
                    VerificationType::Phone => {
                        if let Some(phone) = user.phone_number {
                            match send_verification_sms(&phone, &verification_code.code).await {
                                Ok(_) => Ok(VerificationResponse {
                                    success: true,
                                    message: "Verification code sent to your phone".to_string(),
                                }),
                                Err(e) => {
                                    log::error!("Failed to send verification SMS: {}", e);
                                    Ok(VerificationResponse {
                                        success: false,
                                        message: "Failed to send verification SMS".to_string(),
                                    })
                                }
                            }
                        } else {
                            Ok(VerificationResponse {
                                success: false,
                                message: "No phone number on file".to_string(),
                            })
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create verification code: {:?}", e);
                Ok(VerificationResponse {
                    success: false,
                    message: "Failed to create verification code".to_string(),
                })
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(leptos::ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server]
pub async fn verify_code(
    user_id= i64,
    code: String,
    verification_type: String,
) -> Result<VerificationResponse, leptos::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| leptos::ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let v_type =
            VerificationType::from_str(&verification_type).map_err(|e| leptos::ServerFnError::new(e))?;

        match invitation_database::validate_verification_code(&pool, user_id, &code, v_type).await {
            Ok(true) => Ok(VerificationResponse {
                success: true,
                message: "Verification successful".to_string(),
            }),
            Ok(false) => Ok(VerificationResponse {
                success: false,
                message: "Invalid or expired verification code".to_string(),
            }),
            Err(e) => {
                log::error!("Failed to verify code: {:?}", e);
                Ok(VerificationResponse {
                    success: false,
                    message: "Verification failed".to_string(),
                })
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(leptos::ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[cfg(feature = "ssr")]
async fn send_verification_email(email: &str, code: &str) -> Result<(), String> {
    use crate::app::services::email_service;
    use std::env;

    let app_name = env::var("APP_NAME").unwrap_or_else(|_| "Teapot Testing".to_string());

    // Create verification email content
    let subject = format!("{} - Email Verification Code", app_name);
    let body = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="utf-8">
            <title>Email Verification</title>
        </head>
        <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
            <div style="text-align: center; margin-bottom: 30px;">
                <h1 style="color: #667eea;">🫖 {}</h1>
            </div>
            
            <div style="background: #f8f9fa; padding: 30px; border-radius: 8px; text-align: center;">
                <h2 style="color: #333; margin-bottom: 20px;">Email Verification</h2>
                
                <p style="color: #666; font-size: 16px; margin-bottom: 30px;">
                    Please enter this verification code to confirm your email address:
                </p>
                
                <div style="background: white; border: 2px solid #667eea; border-radius: 8px; padding: 20px; margin: 20px 0; font-size: 32px; font-weight: bold; letter-spacing: 8px; color: #667eea;">
                    {}
                </div>
                
                <p style="color: #999; font-size: 14px;">
                    This code will expire in 10 minutes. If you didn't request this verification, please ignore this email.
                </p>
            </div>
        </body>
        </html>
        "#,
        app_name, code
    );

    // Use existing email service infrastructure
    email_service::send_email(email, &subject, &body).await
}

#[cfg(feature = "ssr")]
async fn send_verification_sms(phone: &str, code: &str) -> Result<(), String> {
    use std::env;

    let app_name = env::var("APP_NAME").unwrap_or_else(|_| "Teapot Testing".to_string());
    let message = format!(
        "Your {} verification code is: {}. This code expires in 10 minutes.",
        app_name, code
    );

    // Check if we're in development mode
    let is_development =
        env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()) != "production";

    if is_development {
        // In development, log the SMS instead of sending it
        log::info!("SMS to {}: {}", phone, message);
        Ok(())
    } else {
        // In production, use actual SMS service
        send_sms_via_service(phone, &message).await
    }
}

#[cfg(feature = "ssr")]
async fn send_sms_via_service(phone: &str, message: &str) -> Result<(), String> {
    use std::env;

    // Get Twilio credentials from environment
    let account_sid = env::var("TWILIO_ACCOUNT_SID").map_err(|_| "TWILIO_ACCOUNT_SID not set")?;
    let auth_token = env::var("TWILIO_AUTH_TOKEN").map_err(|_| "TWILIO_AUTH_TOKEN not set")?;
    let from_phone = env::var("TWILIO_PHONE_NUMBER").map_err(|_| "TWILIO_PHONE_NUMBER not set")?;

    // Create Twilio API request
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        account_sid
    );

    let params = [
        ("From", from_phone.as_str()),
        ("To", phone),
        ("Body", message),
    ];

    let response = client
        .post(&url)
        .basic_auth(&account_sid, Some(&auth_token))
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send SMS request: {}", e))?;

    if response.status().is_success() {
        log::info!("SMS sent successfully to {}", phone);
        Ok(())
    } else {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        log::error!("Failed to send SMS to {}: {}", phone, error_body);
        Err(format!("SMS sending failed: {}", error_body))
    }
}

#[server]
pub async fn get_invitations(
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Invitation>, leptos::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::middleware::authentication;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| leptos::ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let req = extract::<actix_web::HttpRequest>()
            .await
            .map_err(|e| leptos::ServerFnError::new(format!("Failed to extract request: {}", e)))?;

        // Check if user is admin
        match authentication::get_current_user_from_request(&req) {
            Some(user) if user.is_admin() => {
                let limit = limit.unwrap_or(50);
                let offset = offset.unwrap_or(0);

                match invitation_database::get_all_invitations_for_admin(&pool, limit, offset).await
                {
                    Ok(invitations) => Ok(invitations),
                    Err(e) => {
                        log::error!("Failed to get invitations: {:?}", e);
                        Err(leptos::ServerFnError::new(
                            "Failed to fetch invitations".to_string(),
                        ))
                    }
                }
            }
            Some(_) => Err(leptos::ServerFnError::new("Admin access required".to_string())),
            None => Err(leptos::ServerFnError::new("Authentication required".to_string())),
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(leptos::ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server]
pub async fn delete_invitation(invitation_id= i64) -> Result<VerificationResponse, leptos::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::middleware::authentication;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| leptos::ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let req = extract::<actix_web::HttpRequest>()
            .await
            .map_err(|e| leptos::ServerFnError::new(format!("Failed to extract request: {}", e)))?;

        // Check if user is admin
        match authentication::get_current_user_from_request(&req) {
            Some(user) if user.is_admin() => {
                match invitation_database::delete_invitation(&pool, invitation_id).await {
                    Ok(true) => Ok(VerificationResponse {
                        success: true,
                        message: "Invitation deleted successfully".to_string(),
                    }),
                    Ok(false) => Ok(VerificationResponse {
                        success: false,
                        message: "Invitation not found".to_string(),
                    }),
                    Err(e) => {
                        log::error!("Failed to delete invitation: {:?}", e);
                        Ok(VerificationResponse {
                            success: false,
                            message: "Failed to delete invitation".to_string(),
                        })
                    }
                }
            }
            Some(_) => Ok(VerificationResponse {
                success: false,
                message: "Admin access required".to_string(),
            }),
            None => Ok(VerificationResponse {
                success: false,
                message: "Authentication required".to_string(),
            }),
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(leptos::ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[cfg(not(feature = "ssr"))]
async fn send_verification_email(_email: &str, _code: &str) -> Result<(), String> {
    Err("Email sending only available on server".to_string())
}

#[cfg(not(feature = "ssr"))]
async fn send_verification_sms(_phone: &str, _code: &str) -> Result<(), String> {
    Err("SMS sending only available on server".to_string())
}
