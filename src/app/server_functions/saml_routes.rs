use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use crate::app::db::saml_database;
use crate::app::models::user::SessionUser;
use crate::app::db::user_database;

#[derive(Deserialize)]
pub struct SamlAcsRequest {
    #[serde(rename = "SAMLResponse")]
    saml_response: String,
    #[serde(rename = "RelayState")]
    relay_state: Option<String>,
}

#[derive(Deserialize)]
pub struct SamlSloRequest {
    #[serde(rename = "SAMLRequest")]
    saml_request: Option<String>,
    #[serde(rename = "SAMLResponse")]
    saml_response: Option<String>,
    #[serde(rename = "RelayState")]
    relay_state: Option<String>,
}

// Serve SAML metadata for service provider
pub async fn saml_metadata(
    pool: web::Data<sqlx::PgPool>,
) -> Result<HttpResponse> {
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    let metadata_xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata"
                     entityID="{}/saml/metadata">
    <md:SPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
        <md:KeyDescriptor use="signing">
            <ds:KeyInfo xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
                <ds:X509Data>
                    <ds:X509Certificate>
                        <!-- Your SP certificate here -->
                    </ds:X509Certificate>
                </ds:X509Data>
            </ds:KeyInfo>
        </md:KeyDescriptor>
        <md:KeyDescriptor use="encryption">
            <ds:KeyInfo xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
                <ds:X509Data>
                    <ds:X509Certificate>
                        <!-- Your SP certificate here -->
                    </ds:X509Certificate>
                </ds:X509Data>
            </ds:KeyInfo>
        </md:KeyDescriptor>
        <md:SingleLogoutService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect"
                               Location="{}/saml/sls"/>
        <md:NameIDFormat>urn:oasis:names:tc:SAML:2.0:nameid-format:emailAddress</md:NameIDFormat>
        <md:AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
                                    Location="{}/saml/acs"
                                    index="1"/>
    </md:SPSSODescriptor>
</md:EntityDescriptor>"#, base_url, base_url, base_url);

    Ok(HttpResponse::Ok()
        .content_type("application/xml")
        .body(metadata_xml))
}

// Handle SAML Assertion Consumer Service (ACS)
pub async fn saml_acs(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
    form: web::Form<SamlAcsRequest>,
) -> Result<HttpResponse> {
    log::info!("Received SAML ACS request");

    // Get institution ID from cookie
    let institution_id = req
        .cookie("saml_institution")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| {
            log::error!("No institution ID found in cookie");
            actix_web::error::ErrorBadRequest("No institution ID found")
        })?;

    log::info!("Processing SAML response for institution: {}", institution_id);

    // Get SAML config
    let config = match saml_database::get_saml_config(&pool, &institution_id).await {
        Ok(Some(config)) => config,
        Ok(None) => {
            log::error!("No SAML config found for institution: {}", institution_id);
            return Ok(HttpResponse::BadRequest().body("Institution not configured"));
        }
        Err(e) => {
            log::error!("Error getting SAML config: {:?}", e);
            return Ok(HttpResponse::InternalServerError().body("Configuration error"));
        }
    };

    // Decode and parse SAML response
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let saml_manager = match saml_database::SamlManager::new(&base_url) {
        Ok(manager) => manager,
        Err(e) => {
            log::error!("Failed to create SAML manager: {:?}", e);
            return Ok(HttpResponse::InternalServerError().body("SAML configuration error"));
        }
    };

    // Decode base64 SAML response
    use base64::{engine::general_purpose, Engine as _};
    let decoded_response = match general_purpose::STANDARD.decode(&form.saml_response) {
        Ok(decoded) => decoded,
        Err(e) => {
            log::error!("Failed to decode SAML response: {:?}", e);
            return Ok(HttpResponse::BadRequest().body("Invalid SAML response format"));
        }
    };

    let saml_xml = match String::from_utf8(decoded_response) {
        Ok(xml) => xml,
        Err(e) => {
            log::error!("Invalid UTF-8 in SAML response: {:?}", e);
            return Ok(HttpResponse::BadRequest().body("Invalid SAML response encoding"));
        }
    };

    log::debug!("SAML Response XML: {}", saml_xml);

    // Parse SAML response
    let parsed_response = match saml_manager.parse_saml_response(&saml_xml, &institution_id) {
        Ok(response) => response,
        Err(e) => {
            log::error!("Failed to parse SAML response: {:?}", e);
            return Ok(HttpResponse::BadRequest().body("Invalid SAML response"));
        }
    };

    log::info!("Parsed SAML response for user: {}", parsed_response.name_id);

    // Provision or get existing user
    let user = match saml_database::provision_saml_user(&pool, &parsed_response, &institution_id).await {
        Ok(user) => user,
        Err(e) => {
            log::error!("Failed to provision SAML user: {:?}", e);
            return Ok(HttpResponse::InternalServerError().body("User provisioning failed"));
        }
    };

    // Create session
    let session_token = match user_database::create_session(&pool, user.id).await {
        Ok(token) => token,
        Err(e) => {
            log::error!("Failed to create session: {:?}", e);
            return Ok(HttpResponse::InternalServerError().body("Session creation failed"));
        }
    };

    log::info!("SAML login successful for user: {} ({})", user.username, user.id);

    // Build redirect URL
    let redirect_url = form.relay_state.as_deref().unwrap_or("/dashboard");

    // Create HTML response with auto-redirect and session cookie
    let html_response = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Login Successful</title>
    <meta http-equiv="refresh" content="0;url={}">
</head>
<body>
    <p>Login successful! Redirecting...</p>
    <script>
        window.location.href = '{}';
    </script>
</body>
</html>"#, redirect_url, redirect_url);

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .cookie(
            actix_web::cookie::Cookie::build("session", session_token)
                .path("/")
                .http_only(true)
                .same_site(actix_web::cookie::SameSite::Strict)
                .max_age(actix_web::cookie::time::Duration::days(7))
                .secure(!cfg!(debug_assertions))
                .finish()
        )
        .cookie(
            actix_web::cookie::Cookie::build("saml_institution", "")
                .path("/")
                .http_only(true)
                .max_age(actix_web::cookie::time::Duration::seconds(0))
                .finish()
        )
        .body(html_response))
}

// Handle SAML Single Logout Service (SLS)
pub async fn saml_sls(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
    query: web::Query<SamlSloRequest>,
) -> Result<HttpResponse> {
    log::info!("Received SAML SLS request");

    // Get current session
    let session_token = req
        .cookie("session")
        .map(|cookie| cookie.value().to_string());

    if let Some(token) = session_token {
        // Delete the session
        if let Err(e) = user_database::delete_session(&pool, &token).await {
            log::error!("Failed to delete session: {:?}", e);
        }
    }

    // If this is a logout response, just redirect
    if query.saml_response.is_some() {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/login"))
            .cookie(
                actix_web::cookie::Cookie::build("session", "")
                    .path("/")
                    .http_only(true)
                    .max_age(actix_web::cookie::time::Duration::seconds(0))
                    .finish()
            )
            .finish());
    }

    // If this is a logout request, we should process it and send a response
    // For now, just redirect to login page
    let redirect_url = query.relay_state.as_deref().unwrap_or("/login");

    Ok(HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .cookie(
            actix_web::cookie::Cookie::build("session", "")
                .path("/")
                .http_only(true)
                .max_age(actix_web::cookie::time::Duration::seconds(0))
                .finish()
        )
        .finish())
}

// Health check endpoint for SAML functionality
pub async fn saml_health(
    pool: web::Data<sqlx::PgPool>,
) -> Result<HttpResponse> {
    // Check if we can connect to database and have SAML configs
    match saml_database::list_saml_configs(&pool).await {
        Ok(configs) => {
            let active_configs = configs.iter().filter(|c| c.active).count();
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "ok",
                "saml_enabled": true,
                "active_institutions": active_configs,
                "total_institutions": configs.len()
            })))
        }
        Err(e) => {
            log::error!("SAML health check failed: {:?}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "error",
                "saml_enabled": false,
                "error": "Database connectivity issue"
            })))
        }
    }
}

// Configure SAML routes
pub fn configure_saml_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/saml")
            .route("/metadata", web::get().to(saml_metadata))
            .route("/acs", web::post().to(saml_acs))
            .route("/sls", web::get().to(saml_sls))
            .route("/sls", web::post().to(saml_sls))
            .route("/health", web::get().to(saml_health))
    );
}
