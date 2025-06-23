use crate::app::db::saml_database;
use crate::app::models::auth::{AuthProvider, SamlConfig, SamlResponse};
use crate::app::models::user::{SessionUser, UserRole};
use leptos::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use {
    actix_web::{cookie::Cookie, http::header, HttpRequest, HttpResponse},
    leptos_actix::{extract, ResponseOptions},
    sqlx::{PgPool, Row}, // Added Row import
    url::Url,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SamlAuthResponse {
    pub success: bool,
    pub message: String,
    pub redirect_url: Option<String>,
    pub user: Option<SessionUser>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SamlInstitution {
    pub id: String,
    pub name: String,
    pub active: bool,
}
impl SamlInstitution {
    pub fn to_url_safe(&self) -> String {
        self.name
            .to_lowercase()
            .replace(" ", "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
    }
}

// Get list of available SAML institutions
#[server(GetSamlInstitutions, "/api")]
pub async fn get_saml_institutions() -> Result<Vec<SamlInstitution>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Check if user is admin for full list, otherwise return only active configs
        let current_user = crate::app::server_functions::auth::get_current_user().await?;
        let is_admin = current_user
            .map(|u| matches!(u.role, UserRole::Admin | UserRole::SuperAdmin))
            .unwrap_or(false);

        let configs = if is_admin {
            // Admins see all configurations
            saml_database::list_saml_configs(&pool).await?
        } else {
            // Regular users only see active configurations
            let all_configs = saml_database::list_saml_configs(&pool).await?;
            all_configs
                .into_iter()
                .filter(|config| config.active)
                .collect()
        };

        let institutions = configs
            .into_iter()
            .map(|config| SamlInstitution {
                id: config.id.to_string(), // Use UUID as string for management operations
                name: config.institution_name,
                active: config.active,
            })
            .collect();

        Ok(institutions)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Initiate SAML login
#[server(InitiateSamlLogin, "/api")]
pub async fn initiate_saml_login(
    institution_id: String,
    relay_state: Option<String>,
) -> Result<SamlAuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Get SAML config for institution
        let config = saml_database::get_saml_config(&pool, &institution_id)
            .await?
            .ok_or_else(|| {
                ServerFnError::new("Institution not found or not configured for SAML")
            })?;

        // Create SAML manager
        let base_url = std::env::var("BASE_URL").expect("BASE_URL environment variable not set");
        let saml_manager = saml_database::SamlManager::new(&base_url)
            .map_err(|e| ServerFnError::new(format!("Failed to create SAML manager: {}", e)))?;

        // Generate auth request URL
        let auth_url = saml_manager
            .generate_auth_request(&institution_id, relay_state.as_deref())
            .map_err(|e| ServerFnError::new(format!("Failed to generate auth request: {}", e)))?;

        // Store the institution ID in session for later use
        let response = expect_context::<ResponseOptions>();
        let cookie_value = format!(
            "saml_institution={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=600{}",
            institution_id,
            if cfg!(debug_assertions) {
                ""
            } else {
                "; Secure"
            }
        );

        response.insert_header(
            header::SET_COOKIE,
            header::HeaderValue::from_str(&cookie_value).expect("Failed to create header value"),
        );

        Ok(SamlAuthResponse {
            success: true,
            message: "Redirecting to SAML provider".to_string(),
            redirect_url: Some(auth_url),
            user: None,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Handle SAML response (ACS endpoint)
#[server(HandleSamlResponse, "/api")]
pub async fn handle_saml_response(
    saml_response: String,
    relay_state: Option<String>,
) -> Result<SamlAuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use base64::{engine::general_purpose, Engine as _};
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        let req = extract::<HttpRequest>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract request: {}", e)))?;

        // Get institution ID from cookie
        let cookies = req
            .cookies()
            .map_err(|e| ServerFnError::new(format!("Failed to get cookies: {}", e)))?;
        let institution_id = cookies
            .iter()
            .find(|c| c.name() == "saml_institution")
            .map(|c| c.value().to_string())
            .ok_or_else(|| ServerFnError::new("No institution ID found in session"))?;

        // Get SAML config
        let config = saml_database::get_saml_config(&pool, &institution_id)
            .await?
            .ok_or_else(|| ServerFnError::new("Institution not found"))?;

        // Decode SAML response
        let decoded_response = general_purpose::STANDARD
            .decode(&saml_response)
            .map_err(|e| ServerFnError::new(format!("Failed to decode SAML response: {}", e)))?;

        let saml_xml = String::from_utf8(decoded_response)
            .map_err(|e| ServerFnError::new(format!("Invalid UTF-8 in SAML response: {}", e)))?;

        // Create SAML manager and parse response
        let base_url = std::env::var("BASE_URL")
            .expect("BASE_URL environment variable must be set for SAML functionality");
        let saml_manager = saml_database::SamlManager::new(&base_url)
            .map_err(|e| ServerFnError::new(format!("Failed to create SAML manager: {}", e)))?;

        let parsed_response = saml_manager
            .parse_saml_response(&saml_xml, &institution_id)
            .map_err(|e| ServerFnError::new(format!("Failed to parse SAML response: {}", e)))?;

        // Provision or get existing user
        let user =
            saml_database::provision_saml_user(&pool, &parsed_response, &institution_id).await?;

        // Create session
        let session_token = crate::app::db::user_database::create_session(&pool, user.id)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to create session: {}", e)))?;

        // Set session cookie
        let response = expect_context::<ResponseOptions>();
        let session_cookie = format!(
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
            header::HeaderValue::from_str(&session_cookie).expect("Failed to create header value"),
        );

        // Clear institution cookie
        let clear_institution_cookie = format!(
            "saml_institution=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{}",
            if cfg!(debug_assertions) {
                ""
            } else {
                "; Secure"
            }
        );

        response.insert_header(
            header::SET_COOKIE,
            header::HeaderValue::from_str(&clear_institution_cookie)
                .expect("Failed to create header value"),
        );

        // Convert to SessionUser (only include fields that exist in SessionUser)
        let session_user = SessionUser {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            display_name: user.display_name,
            first_name: user.first_name,
            last_name: user.last_name,
        };

        Ok(SamlAuthResponse {
            success: true,
            message: "SAML login successful".to_string(),
            redirect_url: relay_state.or_else(|| Some("/dashboard".to_string())),
            user: Some(session_user),
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Initiate SAML logout
#[server(InitiateSamlLogout, "/api")]
pub async fn initiate_saml_logout() -> Result<SamlAuthResponse, ServerFnError> {
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

        // Get current session
        let cookies = req
            .cookies()
            .map_err(|e| ServerFnError::new(format!("Failed to get cookies: {}", e)))?;
        let session_token = cookies
            .iter()
            .find(|c| c.name() == "session")
            .map(|c| c.value())
            .ok_or_else(|| ServerFnError::new("No active session found"))?;

        // Validate session and get user
        let user = crate::app::db::user_database::validate_session(&pool, session_token)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to validate session: {}", e)))?
            .ok_or_else(|| ServerFnError::new("Invalid session"))?;

        // Check if user has SAML mapping to determine logout URL
        let saml_mapping = sqlx::query(
            "SELECT institution_id, saml_name_id FROM saml_user_mappings WHERE user_id = $1 LIMIT 1"
        )
        .bind(user.id)
        .fetch_optional(&**pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

        if let Some(mapping) = saml_mapping {
            let institution_id: String = mapping.get("institution_id");
            let saml_name_id: String = mapping.get("saml_name_id");

            // Get SAML config
            if let Some(config) = saml_database::get_saml_config(&pool, &institution_id).await? {
                if let Some(slo_url) = config.slo_url {
                    // Generate SAML logout request
                    let base_url = std::env::var("BASE_URL")
                        .expect("BASE_URL environment variable must be set for SAML functionality");
                    let saml_manager = saml_database::SamlManager::new(&base_url).map_err(|e| {
                        ServerFnError::new(format!("Failed to create SAML manager: {}", e))
                    })?;

                    let logout_request = saml_manager
                        .generate_logout_request(&saml_name_id, None)
                        .map_err(|e| {
                            ServerFnError::new(format!("Failed to generate logout request: {}", e))
                        })?;

                    // Create logout URL
                    let mut logout_url = Url::parse(&slo_url)
                        .map_err(|e| ServerFnError::new(format!("Invalid SLO URL: {}", e)))?;
                    logout_url
                        .query_pairs_mut()
                        .append_pair("SAMLRequest", &logout_request);

                    // Delete local session
                    let _ =
                        crate::app::db::user_database::delete_session(&pool, session_token).await;

                    // Clear session cookie
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
                        header::HeaderValue::from_str(&clear_cookie)
                            .expect("Failed to create header value"),
                    );

                    return Ok(SamlAuthResponse {
                        success: true,
                        message: "Redirecting to SAML logout".to_string(),
                        redirect_url: Some(logout_url.to_string()),
                        user: None,
                    });
                }
            }
        }

        // Fallback to local logout if no SAML SLO configured
        let _ = crate::app::db::user_database::delete_session(&pool, session_token).await;

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

        Ok(SamlAuthResponse {
            success: true,
            message: "Local logout successful".to_string(),
            redirect_url: Some("/login".to_string()),
            user: None,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Admin function to create SAML config
#[server(CreateSamlConfig, "/api")]
pub async fn create_saml_config(
    institution_name: String,
    entity_id: String,
    sso_url: String,
    slo_url: Option<String>,
    x509_cert: String,
    metadata_url: Option<String>,
) -> Result<SamlAuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use chrono::Utc;
        use leptos_actix::extract;
        use std::collections::HashMap;
        use uuid::Uuid;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Verify admin permissions
        let current_user = crate::app::server_functions::auth::get_current_user()
            .await?
            .ok_or_else(|| ServerFnError::new("Authentication required"))?;

        if !matches!(current_user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Err(ServerFnError::new("Admin privileges required"));
        }

        let config = SamlConfig {
            id: Uuid::new_v4(),
            institution_name: institution_name.clone(),
            entity_id,
            sso_url,
            slo_url,
            x509_cert,
            metadata_url,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attribute_mapping: HashMap::new(),
            role_mapping: HashMap::new(),
            auto_provision: true,
            require_encrypted_assertions: false,
        };

        saml_database::create_saml_config(&pool, &config).await?;

        Ok(SamlAuthResponse {
            success: true,
            message: format!("SAML configuration created for {}", institution_name),
            redirect_url: None,
            user: None,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Update existing SAML config
#[server(UpdateSamlConfig, "/api")]
pub async fn update_saml_config(
    config_id: String,
    institution_name: String,
    entity_id: String,
    sso_url: String,
    slo_url: Option<String>,
    x509_cert: String,
    metadata_url: Option<String>,
    active: bool,
) -> Result<SamlAuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use uuid::Uuid;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Verify admin permissions
        let current_user = crate::app::server_functions::auth::get_current_user()
            .await?
            .ok_or_else(|| ServerFnError::new("Authentication required"))?;

        if !matches!(current_user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Err(ServerFnError::new("Admin privileges required"));
        }

        let config_uuid = Uuid::parse_str(&config_id)
            .map_err(|_| ServerFnError::new("Invalid config ID format"))?;

        saml_database::update_saml_config(
            &pool,
            config_uuid,
            &institution_name,
            &entity_id,
            &sso_url,
            slo_url.as_deref(),
            &x509_cert,
            metadata_url.as_deref(),
            active,
        )
        .await?;

        Ok(SamlAuthResponse {
            success: true,
            message: format!("SAML configuration updated for {}", institution_name),
            redirect_url: None,
            user: None,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Delete SAML config
#[server(DeleteSamlConfig, "/api")]
pub async fn delete_saml_config(config_id: String) -> Result<SamlAuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use uuid::Uuid;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Verify admin permissions
        let current_user = crate::app::server_functions::auth::get_current_user()
            .await?
            .ok_or_else(|| ServerFnError::new("Authentication required"))?;

        if !matches!(current_user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Err(ServerFnError::new("Admin privileges required"));
        }

        let config_uuid = Uuid::parse_str(&config_id)
            .map_err(|_| ServerFnError::new("Invalid config ID format"))?;

        let institution_name = saml_database::delete_saml_config(&pool, config_uuid).await?;

        Ok(SamlAuthResponse {
            success: true,
            message: format!("SAML configuration deleted for {}", institution_name),
            redirect_url: None,
            user: None,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Toggle SAML config active status
#[server(ToggleSamlConfig, "/api")]
pub async fn toggle_saml_config(config_id: String) -> Result<SamlAuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use uuid::Uuid;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Verify admin permissions
        let current_user = crate::app::server_functions::auth::get_current_user()
            .await?
            .ok_or_else(|| ServerFnError::new("Authentication required"))?;

        if !matches!(current_user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Err(ServerFnError::new("Admin privileges required"));
        }

        let config_uuid = Uuid::parse_str(&config_id)
            .map_err(|_| ServerFnError::new("Invalid config ID format"))?;

        let (institution_name, new_status) =
            saml_database::toggle_saml_config_status(&pool, config_uuid).await?;

        Ok(SamlAuthResponse {
            success: true,
            message: format!(
                "SAML configuration for {} is now {}",
                institution_name,
                if new_status { "active" } else { "inactive" }
            ),
            redirect_url: None,
            user: None,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Get detailed SAML config (for editing)
#[server(GetSamlConfigDetails, "/api")]
pub async fn get_saml_config_details(config_id: String) -> Result<SamlConfig, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use uuid::Uuid;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Verify admin permissions
        let current_user = crate::app::server_functions::auth::get_current_user()
            .await?
            .ok_or_else(|| ServerFnError::new("Authentication required"))?;

        if !matches!(current_user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Err(ServerFnError::new("Admin privileges required"));
        }

        let config_uuid = Uuid::parse_str(&config_id)
            .map_err(|_| ServerFnError::new("Invalid config ID format"))?;

        let config = saml_database::get_saml_config_by_id(&pool, config_uuid)
            .await?
            .ok_or_else(|| ServerFnError::new("SAML configuration not found"))?;

        Ok(config)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Test SAML configuration
#[server(TestSamlConfig, "/api")]
pub async fn test_saml_config(config_id: String) -> Result<SamlAuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use uuid::Uuid;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Verify admin permissions
        let current_user = crate::app::server_functions::auth::get_current_user()
            .await?
            .ok_or_else(|| ServerFnError::new("Authentication required"))?;

        if !matches!(current_user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Err(ServerFnError::new("Admin privileges required"));
        }

        let config_uuid = Uuid::parse_str(&config_id)
            .map_err(|_| ServerFnError::new("Invalid config ID format"))?;

        let config = saml_database::get_saml_config_by_id(&pool, config_uuid)
            .await?
            .ok_or_else(|| ServerFnError::new("SAML configuration not found"))?;

        // Test the configuration by trying to create a SAML manager and generate a test request
        let base_url = std::env::var("BASE_URL")
            .expect("BASE_URL environment variable must be set for SAML functionality");
        let saml_manager = saml_database::SamlManager::new(&base_url)
            .map_err(|e| ServerFnError::new(format!("Failed to create SAML manager: {}", e)))?;

        // Try to validate the certificate format
        if config.x509_cert.trim().is_empty() {
            return Ok(SamlAuthResponse {
                success: false,
                message: "X.509 certificate is required".to_string(),
                redirect_url: None,
                user: None,
            });
        }

        // Test URL connectivity (basic validation)
        if !config.sso_url.starts_with("http") {
            return Ok(SamlAuthResponse {
                success: false,
                message: "SSO URL must be a valid HTTP/HTTPS URL".to_string(),
                redirect_url: None,
                user: None,
            });
        }

        Ok(SamlAuthResponse {
            success: true,
            message: format!(
                "SAML configuration for {} appears to be valid",
                config.institution_name
            ),
            redirect_url: None,
            user: None,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Get SAML statistics
#[server(GetSamlStats, "/api")]
pub async fn get_saml_stats() -> Result<SamlStats, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Verify admin permissions
        let current_user = crate::app::server_functions::auth::get_current_user()
            .await?
            .ok_or_else(|| ServerFnError::new("Authentication required"))?;

        if !matches!(current_user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Err(ServerFnError::new("Admin privileges required"));
        }

        let stats = saml_database::get_saml_statistics(&pool).await?;
        Ok(stats)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SamlStats {
    pub total_institutions: i64,
    pub active_institutions: i64,
    pub total_saml_users: i64,
    pub recent_logins: i64, // Last 30 days
}

// Import metadata from URL
#[server(ImportSamlMetadata, "/api")]
pub async fn import_saml_metadata(
    metadata_url: String,
    institution_name: String,
) -> Result<SamlMetadataResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Verify admin permissions
        let current_user = crate::app::server_functions::auth::get_current_user()
            .await?
            .ok_or_else(|| ServerFnError::new("Authentication required"))?;

        if !matches!(current_user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Err(ServerFnError::new("Admin privileges required"));
        }

        // Validate URL
        if !metadata_url.starts_with("http") {
            return Err(ServerFnError::new(
                "Metadata URL must be a valid HTTP/HTTPS URL",
            ));
        }

        let metadata = saml_database::fetch_and_parse_metadata(&metadata_url).await?;

        Ok(SamlMetadataResponse {
            success: true,
            message: "Metadata imported successfully".to_string(),
            entity_id: metadata.entity_id,
            sso_url: metadata.sso_url,
            slo_url: metadata.slo_url,
            x509_cert: metadata.x509_cert,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SamlMetadataResponse {
    pub success: bool,
    pub message: String,
    pub entity_id: String,
    pub sso_url: String,
    pub slo_url: Option<String>,
    pub x509_cert: String,
}

#[server(GetDetailedSamlInstitutions, "/api")]
pub async fn get_detailed_saml_institutions() -> Result<Vec<SamlInstitutionDetail>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Verify admin permissions
        let current_user = crate::app::server_functions::auth::get_current_user()
            .await?
            .ok_or_else(|| ServerFnError::new("Authentication required"))?;

        if !matches!(current_user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Err(ServerFnError::new("Admin privileges required"));
        }

        let configs = saml_database::get_all_saml_configs_detailed(&pool).await?;

        let institutions = configs
            .into_iter()
            .map(|config| SamlInstitutionDetail {
                id: config.id.to_string(),
                name: config.institution_name,
                entity_id: config.entity_id,
                active: config.active,
                created_at: config.created_at,
                sso_url: config.sso_url,
                has_slo: config.slo_url.is_some(),
                has_metadata_url: config.metadata_url.is_some(),
            })
            .collect();

        Ok(institutions)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SamlInstitutionDetail {
    pub id: String,
    pub name: String,
    pub entity_id: String,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub sso_url: String,
    pub has_slo: bool,
    pub has_metadata_url: bool,
}

// Add a bulk operations function for admin convenience
#[server(BulkToggleSamlConfigs, "/api")]
pub async fn bulk_toggle_saml_configs(
    config_ids: Vec<String>,
    enable: bool,
) -> Result<SamlAuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        use uuid::Uuid;

        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        // Verify admin permissions
        let current_user = crate::app::server_functions::auth::get_current_user()
            .await?
            .ok_or_else(|| ServerFnError::new("Authentication required"))?;

        if !matches!(current_user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Err(ServerFnError::new("Admin privileges required"));
        }

        let mut updated_count = 0;
        let mut errors = Vec::new();

        for config_id in config_ids {
            match Uuid::parse_str(&config_id) {
                Ok(uuid) => {
                    match saml_database::update_saml_config_status(&pool, uuid, enable).await {
                        Ok(_) => updated_count += 1,
                        Err(e) => errors.push(format!("Failed to update {}: {}", config_id, e)),
                    }
                }
                Err(_) => errors.push(format!("Invalid UUID format: {}", config_id)),
            }
        }

        let message = if errors.is_empty() {
            format!(
                "Successfully {} {} SAML configuration(s)",
                if enable { "enabled" } else { "disabled" },
                updated_count
            )
        } else {
            format!(
                "Updated {} configurations with {} errors: {}",
                updated_count,
                errors.len(),
                errors.join(", ")
            )
        };

        Ok(SamlAuthResponse {
            success: errors.is_empty(),
            message,
            redirect_url: None,
            user: None,
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

// Add validation function for SAML configurations
#[server(ValidateSamlConfig, "/api")]
pub async fn validate_saml_config(
    entity_id: String,
    sso_url: String,
    slo_url: Option<String>,
    x509_cert: String,
    metadata_url: Option<String>,
) -> Result<SamlValidationResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        // Verify admin permissions
        let current_user = crate::app::server_functions::auth::get_current_user()
            .await?
            .ok_or_else(|| ServerFnError::new("Authentication required"))?;

        if !matches!(current_user.role, UserRole::Admin | UserRole::SuperAdmin) {
            return Err(ServerFnError::new("Admin privileges required"));
        }

        let mut validation_errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate URLs
        if !sso_url.starts_with("https://") && !sso_url.starts_with("http://") {
            validation_errors.push("SSO URL must be a valid HTTP/HTTPS URL".to_string());
        }

        if sso_url.starts_with("http://") && !sso_url.contains("localhost") {
            warnings.push(
                "Using HTTP instead of HTTPS for SSO URL is not recommended for production"
                    .to_string(),
            );
        }

        if let Some(ref slo_url) = slo_url {
            if !slo_url.starts_with("https://") && !slo_url.starts_with("http://") {
                validation_errors.push("SLO URL must be a valid HTTP/HTTPS URL".to_string());
            }
        }

        if let Some(ref metadata_url) = metadata_url {
            if !metadata_url.starts_with("https://") && !metadata_url.starts_with("http://") {
                validation_errors.push("Metadata URL must be a valid HTTP/HTTPS URL".to_string());
            }
        }

        // Validate certificate format
        if !x509_cert.contains("-----BEGIN CERTIFICATE-----")
            || !x509_cert.contains("-----END CERTIFICATE-----")
        {
            validation_errors.push("X.509 certificate must be in PEM format".to_string());
        }

        // Validate entity ID format
        if entity_id.trim().is_empty() {
            validation_errors.push("Entity ID cannot be empty".to_string());
        }

        // Additional checks
        if entity_id.len() > 512 {
            validation_errors.push("Entity ID is too long (max 512 characters)".to_string());
        }

        if sso_url.len() > 2048 {
            validation_errors.push("SSO URL is too long (max 2048 characters)".to_string());
        }

        let is_valid = validation_errors.is_empty();

        Ok(SamlValidationResponse {
            valid: is_valid,
            errors: validation_errors,
            warnings,
            message: if is_valid {
                "Configuration is valid".to_string()
            } else {
                "Configuration has validation errors".to_string()
            },
        })
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SamlValidationResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub message: String,
}
