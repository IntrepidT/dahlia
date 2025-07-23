// All email functionality is only available on the server side
#[cfg(feature = "ssr")]
use lettre::{
    transport::smtp::authentication::Credentials,
    transport::smtp::client::{Tls, TlsParameters},
    Message, SmtpTransport, Transport,
};
#[cfg(feature = "ssr")]
use log::{error, info, warn};
#[cfg(feature = "ssr")]
use std::env;
#[cfg(feature = "ssr")]
use std::time::Duration;

#[cfg(feature = "ssr")]
pub async fn send_reset_email(email: &str, reset_token: &str) -> Result<(), String> {
    // TeaPot Testing email configuration
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port = env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse::<u16>()
        .unwrap_or(587);

    let smtp_username = env::var("SMTP_USERNAME")
        .map_err(|_| "SMTP_USERNAME (noreply@teapottesting.com) must be set".to_string())?;

    let smtp_password = env::var("SMTP_PASSWORD")
        .map_err(|_| "SMTP_PASSWORD (Google App Password) must be set".to_string())?;

    let app_url = env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let from_email =
        env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@teapottesting.com".to_string());

    // Validate configuration
    if !from_email.ends_with("@teapottesting.com") {
        warn!("FROM_EMAIL should use @teapottesting.com domain");
    }

    // Clean password (remove spaces if present) and validate
    let password_clean = smtp_password.replace(" ", "");
    if password_clean.len() != 16 {
        warn!("SMTP password should be 16 characters (Google App Password format). Current length: {}", password_clean.len());
    }

    // Create the reset link
    let reset_link = format!("{}/reset-password/{}", app_url, reset_token);

    info!(
        "Creating password reset email for {} from TeaPot Testing",
        email
    );

    // Create branded email for TeaPot Testing
    let email_message = Message::builder()
        .from(format!("TeaPot Testing <{}>", from_email).parse()
            .map_err(|e| format!("Invalid from email: {}", e))?)
        .to(email.parse().map_err(|e| format!("Invalid recipient email: {}", e))?)
        .subject("TeaPot Testing - Password Reset Request")
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Password Reset - TeaPot Testing</title>
            </head>
            <body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; background-color: #f9f9f9;">
                
                <!-- Header with TeaPot Testing branding -->
                <div style="text-align: center; margin-bottom: 30px;">
                    <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 8px 8px 0 0;">
                        <h1 style="margin: 0; font-size: 28px; font-weight: 300;">🫖 TeaPot Testing</h1>
                        <p style="margin: 5px 0 0 0; opacity: 0.9; font-size: 14px;">Quality Assurance Platform</p>
                    </div>
                </div>

                <!-- Main content -->
                <div style="background-color: white; padding: 30px; border-radius: 0 0 8px 8px; box-shadow: 0 4px 20px rgba(0,0,0,0.1);">
                    <h2 style="color: #667eea; margin-top: 0; font-size: 24px; text-align: center;">Password Reset Request</h2>
                    
                    <p style="font-size: 16px; margin-bottom: 20px;">Hello,</p>
                    
                    <p style="font-size: 16px; margin-bottom: 25px;">We received a request to reset the password for your TeaPot Testing account. Click the button below to set a new password:</p>
                    
                    <!-- Reset button -->
                    <div style="text-align: center; margin: 35px 0;">
                        <a href="{}" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 15px 35px; text-decoration: none; border-radius: 25px; display: inline-block; font-weight: 600; font-size: 16px; box-shadow: 0 4px 15px rgba(102, 126, 234, 0.3);">
                            Reset Your Password
                        </a>
                    </div>
                    
                    <!-- Alternative link -->
                    <div style="margin: 30px 0; padding: 20px; background-color: #f8f9fa; border-radius: 6px; border-left: 4px solid #667eea;">
                        <p style="font-size: 14px; color: #666; margin: 0 0 10px 0; font-weight: 600;">Can't click the button? Copy and paste this link:</p>
                        <div style="word-break: break-all; font-family: monospace; font-size: 14px; background-color: white; padding: 10px; border-radius: 4px; border: 1px solid #e9ecef;">
                            <a href="{0}" style="color: #667eea; text-decoration: none;">{0}</a>
                        </div>
                    </div>
                    
                    <!-- Security notice -->
                    <div style="margin: 30px 0; padding: 20px; background-color: #fff3cd; border-radius: 6px; border-left: 4px solid #ffc107;">
                        <p style="font-size: 14px; color: #856404; margin: 0; font-weight: 600;">🔒 Security Notice</p>
                        <ul style="font-size: 14px; color: #856404; margin: 10px 0 0 0; padding-left: 20px;">
                            <li>This link will expire in <strong>24 hours</strong></li>
                            <li>If you didn't request this reset, please ignore this email</li>
                            <li>For security concerns, contact our support team</li>
                        </ul>
                    </div>
                    
                    <!-- Footer -->
                    <div style="margin-top: 40px; padding-top: 25px; border-top: 2px solid #f1f3f4; text-align: center;">
                        <p style="font-size: 16px; margin: 0 0 15px 0; color: #333;">Happy Testing!</p>
                        <p style="font-size: 16px; margin: 0; font-weight: 600; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;">The TeaPot Testing Team</p>
                    </div>
                </div>
                
                <!-- Email footer -->
                <div style="text-align: center; font-size: 12px; color: #999; margin-top: 25px; padding: 20px;">
                    <p style="margin: 0 0 5px 0;">This is an automated email from TeaPot Testing</p>
                    <p style="margin: 0 0 5px 0;">Please do not reply to this email address</p>
                    <p style="margin: 0;">Need help? Contact us at <a href="mailto:support@teapottesting.com" style="color: #667eea;">support@teapottesting.com</a></p>
                    <hr style="border: none; border-top: 1px solid #eee; margin: 15px 0;">
                    <p style="margin: 0; opacity: 0.7;">© 2025 TeaPot Testing. All rights reserved.</p>
                </div>
            </body>
            </html>
            "#,
            reset_link
        ))
        .map_err(|e| format!("Failed to create email: {}", e))?;

    info!(
        "Sending TeaPot Testing password reset email to {} via Google Workspace ({}:{})",
        email, smtp_server, smtp_port
    );

    let creds = Credentials::new(smtp_username.clone(), smtp_password);

    // Configure TLS for Google Workspace
    let tls_parameters = TlsParameters::new(smtp_server.clone())
        .map_err(|e| format!("TLS configuration error: {}", e))?;

    let mailer = SmtpTransport::builder_dangerous(&smtp_server)
        .port(smtp_port)
        .credentials(creds)
        .tls(Tls::Required(tls_parameters))
        .timeout(Some(Duration::from_secs(30)))
        .build();

    // Send the email
    match mailer.send(&email_message) {
        Ok(response) => {
            info!(
                "✅ TeaPot Testing password reset email sent successfully to {}",
                email
            );
            info!("SMTP Response: {:?}", response);
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to send TeaPot Testing email to {}: {}", email, e);

            // Provide helpful error messages based on error type
            let error_msg = if e.to_string().contains("authentication")
                || e.to_string().contains("Authentication")
            {
                format!("Authentication failed: {}. Check your Google Workspace app password for noreply@teapottesting.com", e)
            } else if e.to_string().contains("timeout") || e.to_string().contains("Timeout") {
                format!(
                    "Connection timeout: {}. Check internet connection and firewall settings.",
                    e
                )
            } else if e.to_string().contains("connection") || e.to_string().contains("Connection") {
                format!("Network error: {}. Check internet connection.", e)
            } else {
                format!("Email sending error: {}", e)
            };

            Err(error_msg)
        }
    }
}

// Provide a stub for client-side builds
#[cfg(not(feature = "ssr"))]
pub async fn send_reset_email(_email: &str, _reset_token: &str) -> Result<(), String> {
    Err("Email sending is only available on the server".to_string())
}
