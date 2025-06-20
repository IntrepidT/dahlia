use crate::app::server_functions::auth::{get_current_user, login, logout, register};
use leptos::*;
use leptos_router::use_navigate;
use log::{debug, error, log};
use serde::Serialize;
#[cfg(feature = "ssr")]
use {
    lettre::transport::smtp::authentication::Credentials,
    lettre::{message::Message, SmtpTransport, Transport},
};

#[derive(Serialize)]
struct EmailContext {
    reset_link: String,
    // Add more fields as needed for your template
}

#[cfg(feature = "ssr")]
pub async fn send_reset_email(email: &str, reset_token: &str) -> Result<(), String> {
    use reqwest::Client;
    use serde_json::{json, Value};

    // Configuration - in production these should come from environment variables
    let sendgrid_api_key = std::env::var("SENDGRID_API_KEY")
        .map_err(|_| "SENDGRID_API_KEY environment variable not set".to_string())?;
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "https://yourapp.com".to_string());
    let from_email =
        std::env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@yourapp.com".to_string());

    // Determine whether to use sandbox mode based on environment
    let is_development =
        std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()) != "production";

    // Create the reset link
    let reset_link = format!("{}/reset-password/{}", app_url, reset_token);

    // Build the SendGrid API request payload
    let mut payload = json!({
        "personalizations": [{
            "to": [{ "email": email }]
        }],
        "from": { "email": from_email },
        "subject": "Password Reset Instructions",
        "content": [{
            "type": "text/plain",
            "value": format!(
                "Click the link below to reset your password:\n\n{}\n\nThis link will expire in 24 hours.",
                reset_link
            )
        }]
    });

    // Only enable sandbox mode for development environment
    if is_development {
        // Add sandbox mode setting for development
        if let Some(payload_obj) = payload.as_object_mut() {
            payload_obj.insert(
                "mail_settings".to_string(),
                json!({
                    "sandbox_mode": {
                        "enable": true
                    }
                }),
            );
            log::info!("Sending password reset email to {} (sandbox mode)", email);
        }
    } else {
        log::info!(
            "Sending password reset email to {} (production mode)",
            email
        );
    }

    // Send the request to SendGrid API
    let client = Client::new();
    let res = client
        .post("https://api.sendgrid.com/v3/mail/send")
        .header("Authorization", format!("Bearer {}", sendgrid_api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to SendGrid: {}", e))?;

    // Check the response
    if res.status().is_success() {
        if is_development {
            log::info!(
                "Password reset email sent successfully to {} (sandbox mode)",
                email
            );
        } else {
            log::info!("Password reset email sent successfully to {}", email);
        }
        Ok(())
    } else {
        let status = res.status();
        let body = res
            .text()
            .await
            .unwrap_or_else(|_| "No response body".to_string());
        error!("Failed to send email. Status: {}, Body: {}", status, body);
        Err(format!("Failed to send email. Status: {}", status))
    }
}
