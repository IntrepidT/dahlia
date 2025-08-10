use leptos::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::app::db::{saml_database, user_database};
        use crate::app::models::user::SessionUser;
        use leptos::html::form as leptos_form;
        use actix_web::{web, HttpRequest, HttpResponse, Result};
        use flate2::{write::DeflateEncoder, Compression};
        use std::io::Write;
        use serde::Deserialize;

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

        #[derive(Deserialize)]
        pub struct SamlLoginQuery {
            institution: String,
            relay_state: Option<String>,
        }

        // Initiate SAML login flow
        pub async fn saml_login(
            pool: web::Data<sqlx::PgPool>,
            query: web::Query<SamlLoginQuery>,
        ) -> Result<HttpResponse> {
            log::info!("Initiating SAML login for institution: {}", query.institution);

            // Convert institution name to match database format
            let institution_name = query
                .institution
                .split('-')
                .map(|word| {
                    if word.to_lowercase() == "saml" {
                        "SAML".to_string()
                    } else {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            log::info!("Converted '{}' to '{}'", query.institution, institution_name);

            // Get SAML config for institution
            let config = match saml_database::get_saml_config_by_name(&pool, &institution_name).await {
                Ok(Some(config)) => config,
                Ok(None) => {
                    log::error!("No SAML config found for institution: {}", institution_name);
                    return Ok(HttpResponse::BadRequest().body(format!(
                        "Institution '{}' not configured for SAML SSO",
                        institution_name
                    )));
                }
                Err(e) => {
                    log::error!("Error getting SAML config: {:?}", e);
                    return Ok(HttpResponse::InternalServerError().body("Configuration error"));
                }
            };

            if !config.active {
                log::warn!("SAML config is disabled for institution: {}", institution_name);
                return Ok(HttpResponse::BadRequest().body("SAML SSO is disabled for this institution"));
            }

            // Create SAML manager
            let base_url = std::env::var("BASE_URL").expect("BASE_URL environment variable must be set for SAML functionality");
            let saml_manager = match saml_database::SamlManager::new(&base_url) {
                Ok(manager) => manager,
                Err(e) => {
                    log::error!("Failed to create SAML manager: {:?}", e);
                    return Ok(HttpResponse::InternalServerError().body("SAML configuration error"));
                }
            };

            // Generate SAML auth request - institution info will be embedded in RelayState
            let auth_url = match generate_auth_request_url(&saml_manager, &config, query.relay_state.as_deref()) {
                Ok(url) => url,
                Err(e) => {
                    log::error!("Failed to generate SAML auth request: {:?}", e);
                    return Ok(HttpResponse::InternalServerError().body("Failed to generate SAML request"));
                }
            };

            log::info!("Redirecting to SAML IdP: {}", auth_url);

            // Simple redirect without cookies
            Ok(HttpResponse::Found()
                .append_header(("Location", auth_url))
                .finish())
        }

// Replace your generate_auth_request_url function with this version:

    fn generate_auth_request_url(
        saml_manager: &saml_database::SamlManager,
        config: &crate::app::models::auth::SamlConfig,
        relay_state: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use uuid::Uuid;
        use chrono::Utc;
        use base64::{engine::general_purpose, Engine as _};
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;

        // Build the SAML AuthnRequest with proper XML formatting
        let request_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");

        let saml_request = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
    <samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="{}" Version="2.0" IssueInstant="{}" Destination="{}" ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" AssertionConsumerServiceURL="{}">
        <saml:Issuer>{}</saml:Issuer>
    </samlp:AuthnRequest>"#,
            request_id,
            timestamp,
            config.sso_url,
            format!("{}/saml/acs", saml_manager.base_url),
            format!("{}/saml/metadata", saml_manager.base_url)
        );

        log::debug!("Generated SAML Request XML: {}", saml_request);

        // Compress and encode the request
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(saml_request.as_bytes())?;
        let compressed_data = encoder.finish()?;
        let encoded_request = general_purpose::STANDARD.encode(&compressed_data);

        // Build the redirect URL
        let mut auth_url = url::Url::parse(&config.sso_url)?;
        auth_url
            .query_pairs_mut()
            .append_pair("SAMLRequest", &encoded_request);

        // IMPORTANT: Encode institution info in RelayState
        // Format: "institution_id|original_relay_state"
        let encoded_relay_state = match relay_state {
            Some(original_relay_state) => {
                format!("{}|{}", config.institution_name, original_relay_state)
            }
            None => {
                format!("{}|/dashboard", config.institution_name)
            }
        };

        auth_url
            .query_pairs_mut()
            .append_pair("RelayState", &encoded_relay_state);

        log::info!("Generated auth URL: {}", auth_url);

        Ok(auth_url.to_string())
    }

        // Serve SAML metadata for service provider
        pub async fn saml_metadata(_pool: web::Data<sqlx::PgPool>) -> Result<HttpResponse> {
            let base_url =
                std::env::var("BASE_URL").expect("BASE_URL environment variable must be set for SAML functionality");

            let metadata_xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
        <md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata"
                             entityID="{}/saml/metadata">
            <md:SPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
                <md:NameIDFormat>urn:oasis:names:tc:SAML:2.0:nameid-format:emailAddress</md:NameIDFormat>
                <md:AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
                                            Location="{}/saml/acs"
                                            index="1"/>
                <md:SingleLogoutService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect"
                                       Location="{}/saml/sls"/>
            </md:SPSSODescriptor>
        </md:EntityDescriptor>"#,
                base_url, base_url, base_url
            );

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

            // Debug: Log all cookies (for debugging purposes)
            if let Ok(cookies) = req.cookies() {
                log::info!("Available cookies:");
                for cookie in cookies.iter() {
                    log::info!("  Cookie: {} = {}", cookie.name(), cookie.value());
                }
            }

            // Extract institution info from RelayState instead of cookies
            let (institution_name, redirect_url) = match &form.relay_state {
                Some(relay_state) if !relay_state.is_empty() => {
                    log::info!("Processing RelayState: {}", relay_state);

                    if relay_state.contains('|') {
                        let parts: Vec<&str> = relay_state.split('|').collect();
                        if parts.len() >= 2 {
                            let institution = parts[0].to_string();
                            let redirect = parts[1].to_string();
                            log::info!("Extracted institution: '{}', redirect: '{}'", institution, redirect);
                            (institution, redirect)
                        } else {
                            log::error!("Invalid RelayState format: {}", relay_state);
                            return Ok(HttpResponse::BadRequest().body("Invalid RelayState format"));
                        }
                    } else {
                        // Fallback: treat entire RelayState as institution name
                        log::warn!("RelayState doesn't contain separator, treating as institution name");
                        (relay_state.clone(), "/dashboard".to_string())
                    }
                }
                _ => {
                    log::error!("No RelayState provided in SAML response");
                    return Ok(HttpResponse::BadRequest().body("No institution information found"));
                }
            };

            log::info!("Processing SAML response for institution: {}", institution_name);

            // Get SAML config - institution_name is already in the correct format from RelayState
            let config = match saml_database::get_saml_config_by_name(&pool, &institution_name).await {
                Ok(Some(config)) => config,
                Ok(None) => {
                    log::error!("No SAML config found for institution: {}", institution_name);
                    return Ok(HttpResponse::BadRequest().body("Institution not configured"));
                }
                Err(e) => {
                    log::error!("Error getting SAML config: {:?}", e);
                    return Ok(HttpResponse::InternalServerError().body("Configuration error"));
                }
            };

            // Rest of your SAML processing logic remains the same...
            // Decode and parse SAML response
            let base_url = std::env::var("BASE_URL").expect("BASE_URL environment variable msut be set for SAML functionality");
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
            let parsed_response = match saml_manager.parse_saml_response(&saml_xml, &institution_name) {
                Ok(response) => response,
                Err(e) => {
                    log::error!("Failed to parse SAML response: {:?}", e);
                    return Ok(HttpResponse::BadRequest().body("Invalid SAML response"));
                }
            };

            log::info!("Parsed SAML response for user: {}", parsed_response.name_id);

            // Provision or get existing user
            let user = match saml_database::provision_saml_user(&pool, &parsed_response, &institution_name).await {
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

            // Use the redirect_url extracted from RelayState
            log::info!("Redirecting to: {}", redirect_url);

            // Create HTML response with auto-redirect and session cookie
            let html_response = format!(
                r#"<!DOCTYPE html>
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
        </html>"#,
                redirect_url, redirect_url
            );

            Ok(HttpResponse::Ok()
                .content_type("text/html")
                .cookie(
                    actix_web::cookie::Cookie::build("session", session_token)
                        .path("/")
                        .http_only(true)
                        .same_site(actix_web::cookie::SameSite::Strict)
                        .max_age(actix_web::cookie::time::Duration::days(7))
                        .secure(true)
                        .finish(),
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
                            .finish(),
                    )
                    .finish());
            }

            // If this is a logout request, we should process it and send a response
            // For now, just redirect to login page
            let redirect_url = match query.relay_state.as_deref() {
                Some(relay_state) if !relay_state.is_empty() && relay_state != "undefined" && relay_state != "null" => {
                    // If using the RelayState encoding method:
                    if relay_state.contains('|') {
                        let parts: Vec<&str> = relay_state.split('|').collect();
                        if parts.len() >= 2 {
                            parts[1].to_string()  // Get the redirect part
                        } else {
                            "/dashboard".to_string()
                        }
                    } else {
                        relay_state.to_string()
                    }
                }
                _ => "/dashboard".to_string()  // Default fallback
            };

            log::info!("Redirecting to: {}", redirect_url);

            // Create HTML response with auto-redirect and session cookie
            let html_response = format!(
                r#"
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
            </html>"#,
                redirect_url, redirect_url  // Use redirect_url instead of form.relay_state
            );

            Ok(HttpResponse::Found()
                .append_header(("Location", redirect_url))
                .cookie(
                    actix_web::cookie::Cookie::build("session", "")
                        .path("/")
                        .http_only(true)
                        .max_age(actix_web::cookie::time::Duration::seconds(0))
                        .finish(),
                )
                .finish())
        }

        // Health check endpoint for SAML functionality
        pub async fn saml_health(pool: web::Data<sqlx::PgPool>) -> Result<HttpResponse> {
            // Check if we can connect to database and have SAML configs
            match saml_database::list_saml_configs(&pool).await {
                Ok(configs) => {
                    let active_configs = configs.iter().filter(|c| c.active).count();
                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "status": "ok",
                        "saml_enabled": true,
                        "active_institutions": active_configs,
                        "total_institutions": configs.len(),
                        "institutions": configs.iter().map(|c| &c.institution_name).collect::<Vec<_>>()
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
                    .route("/login", web::get().to(saml_login))
                    .route("/metadata", web::get().to(saml_metadata))
                    .route("/acs", web::post().to(saml_acs))
                    .route("/sls", web::get().to(saml_sls))
                    .route("/sls", web::post().to(saml_sls))
                    .route("/health", web::get().to(saml_health)),
            );
        }
    }
}

// Non-SSR placeholder for client-side compilation
#[cfg(not(feature = "ssr"))]
pub fn configure_saml_routes(_cfg: &mut ()) {
    // This function exists only for client-side compilation compatibility
    // The actual implementation is behind the SSR feature gate
}

//validates that the production environment is correctly configured for SAML
#[cfg(feature = "ssr")]
fn validate_production_config() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("BASE_URL")?;

    if !base_url.starts_with("https://") {
        return Err("BASE_URL must use HTTPS in production".into());
    }

    if base_url.contains("localhost") || base_url.contains("127.0.0.1") {
        log::warn!("BASE_URL appears to be localhost - ensure this is intentional");
    }

    // Validate that required environment variables are set
    if std::env::var("DATABASE_URL").is_err() {
        return Err("DATABASE_URL must be set".into());
    }

    log::info!("Production SAML configuration validated successfully");
    Ok(())
}

// Call this in your main.rs or app startup:
#[cfg(feature = "ssr")]
pub fn initialize_saml() -> Result<(), Box<dyn std::error::Error>> {
    validate_production_config()?;
    log::info!("SAML subsystem initialized for production");
    Ok(())
}
