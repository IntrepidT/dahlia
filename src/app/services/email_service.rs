#[cfg(feature = "ssr")]
use lettre::{
    transport::smtp::authentication::Credentials,
    transport::smtp::client::{Tls, TlsParameters},
    Message, SmtpTransport, Transport,
};
use log::{error, info};
use std::env;

#[cfg(feature = "ssr")]
pub async fn send_reset_email(email: &str, reset_token: &str) -> Result<(), String> {
    // Configuration - get values from environment variables
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.sendgrid.net".to_string());
    let smtp_port = env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse::<u16>()
        .unwrap_or(587);
    
    // For SendGrid, the username is always "apikey"
    let smtp_username = env::var("SMTP_USERNAME").unwrap_or_else(|_| "apikey".to_string());
    
    // The password is your SendGrid API key
    let smtp_password = env::var("SMTP_PASSWORD")
        .expect("SMTP_PASSWORD (SendGrid API key) must be set");
    
    let app_url = env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let from_email = env::var("FROM_EMAIL")
        .expect("FROM_EMAIL must be set to a verified sender");

    // Create the reset link
    let reset_link = format!("{}/reset-password/{}", app_url, reset_token);

    info!("Creating password reset email for {}", email);

    // Create the email with a more professional template
    let email_message = Message::builder()
        .from(from_email.parse().map_err(|e| format!("Invalid from email: {}", e))?)
        .to(email.parse().map_err(|e| format!("Invalid recipient email: {}", e))?)
        .subject("Password Reset Request")
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
                <div style="background-color: #f7f7f7; padding: 20px; border-radius: 5px; border-top: 4px solid #4285f4;">
                    <h2 style="color: #4285f4; margin-top: 0;">Password Reset Request</h2>
                    <p>Hello,</p>
                    <p>We received a request to reset your password. Click the button below to set a new password:</p>
                    <p style="text-align: center; margin: 30px 0;">
                        <a href="{}" style="background-color: #4285f4; color: white; padding: 12px 24px; text-decoration: none; border-radius: 3px; display: inline-block; font-weight: bold;">Reset Password</a>
                    </p>
                    <p>Or copy and paste this link into your browser:</p>
                    <p style="background-color: #eaeaea; padding: 10px; border-radius: 3px; word-break: break-all;"><a href="{0}">{0}</a></p>
                    <p>This link will expire in 24 hours.</p>
                    <p>If you did not request a password reset, please ignore this email or contact support if you have concerns.</p>
                    <p>Regards,<br>Your App Team</p>
                </div>
                <div style="text-align: center; font-size: 12px; color: #999; margin-top: 20px;">
                    <p>This is an automated email, please do not reply.</p>
                </div>
            </body>
            </html>
            "#,
            reset_link
        ))
        .map_err(|e| format!("Failed to create email: {}", e))?;

    info!(
        "Sending password reset email to {} via SMTP server {}",
        email, smtp_server
    );

    let creds = Credentials::new(smtp_username, smtp_password);

    // Configure TLS for SendGrid
    let tls_parameters =
        TlsParameters::new(smtp_server.clone()).map_err(|e| format!("TLS error: {}", e))?;

    // Open a connection to the SendGrid SMTP server
    let mailer = SmtpTransport::builder_dangerous(&smtp_server)
        .port(smtp_port)
        .credentials(creds)
        .tls(Tls::Required(tls_parameters))
        .build();

    // Send the email
    match mailer.send(&email_message) {
        Ok(_) => {
            info!("Password reset email sent successfully to {}", email);
            Ok(())
        }
        Err(e) => {
            error!("Failed to send email: {}", e);
            Err(format!("Failed to send email: {}", e))
        }
    }
}
