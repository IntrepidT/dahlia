cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Pool, Postgres, Row};
        use leptos::ServerFnError;
        use crate::app::models::auth::{SamlConfig, SamlResponse, AuthProvider};
        use crate::app::models::user::{SessionUser, UserRole, AccountStatus};
        use crate::app::db::user_database;
        use uuid::Uuid;
        use std::collections::HashMap;
        use roxmltree::Document;
        use base64::{Engine as _, engine::general_purpose};
        use chrono::{DateTime, Utc};

        pub struct SamlManager {
            pub base_url: String,
        }

        impl SamlManager {
            pub fn new(base_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
                Ok(SamlManager {
                    base_url: base_url.to_string(),
                })
            }

            pub fn generate_auth_request(&self, institution_id: &str, relay_state: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
                // Generate a simple SAML AuthnRequest
                let request_id = format!("_{}", uuid::Uuid::new_v4().simple());
                let issue_instant = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");

                let auth_request_xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
            <samlp:AuthnRequest 
                xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
                xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
                ID="{}"
                Version="2.0"
                IssueInstant="{}"
                Destination=""
                AssertionConsumerServiceURL="{}/saml/acs"
                ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST">
                <saml:Issuer>{}/saml/metadata</saml:Issuer>
                <samlp:NameIDPolicy 
                    Format="urn:oasis:names:tc:SAML:2.0:nameid-format:emailAddress"
                    AllowCreate="true"/>
            </samlp:AuthnRequest>"#, 
                    request_id,
                    issue_instant,
                    self.base_url,
                    self.base_url
                );

                // Base64 encode the request
                let encoded = general_purpose::STANDARD.encode(&auth_request_xml);

                // Create redirect URL - this should be customized based on your IdP
                let mut url = url::Url::parse(&format!("{}/sso", self.base_url))?;
                url.query_pairs_mut()
                    .append_pair("SAMLRequest", &encoded);

                if let Some(state) = relay_state {
                    url.query_pairs_mut().append_pair("RelayState", state);
                }

                Ok(url.to_string())
            }

            pub fn parse_saml_response(&self, saml_xml: &str, institution_id: &str) -> Result<SamlResponse, Box<dyn std::error::Error>> {
                let doc = Document::parse(saml_xml)?;

                // Extract NameID
                let name_id = doc
                    .descendants()
                    .find(|n| n.has_tag_name("NameID") || n.has_tag_name("saml:NameID"))
                    .and_then(|n| n.text())
                    .ok_or("NameID not found in SAML response")?
                    .to_string();

                // Extract attributes
                let mut attributes = HashMap::new();
                for attr_node in doc.descendants().filter(|n| n.has_tag_name("Attribute") || n.has_tag_name("saml:Attribute")) {
                    if let Some(name) = attr_node.attribute("Name") {
                        let values: Vec<String> = attr_node
                            .descendants()
                            .filter(|n| n.has_tag_name("AttributeValue") || n.has_tag_name("saml:AttributeValue"))
                            .filter_map(|n| n.text())
                            .map(|s| s.to_string())
                            .collect();
                        if !values.is_empty() {
                            attributes.insert(name.to_string(), values);
                        }
                    }
                }

                // Extract common attributes with fallbacks
                let email = self.extract_attribute(&attributes, &[
                    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress",
                    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name",
                    "mail",
                    "email",
                    "emailAddress"
                ]).or_else(|| {
                    // If no email attribute, try to use NameID if it looks like an email
                    if name_id.contains('@') { Some(name_id.clone()) } else { None }
                });

                let first_name = self.extract_attribute(&attributes, &[
                    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname",
                    "givenName",
                    "firstName",
                    "given_name"
                ]);

                let last_name = self.extract_attribute(&attributes, &[
                    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname",
                    "sn",
                    "surname",
                    "lastName",
                    "last_name",
                    "familyName"
                ]);

                let display_name = self.extract_attribute(&attributes, &[
                    "http://schemas.microsoft.com/identity/claims/displayname",
                    "displayName",
                    "cn",
                    "commonName"
                ]).or_else(|| {
                    // Construct display name from first and last name if available
                    match (first_name.as_ref(), last_name.as_ref()) {
                        (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                        (Some(first), None) => Some(first.clone()),
                        (None, Some(last)) => Some(last.clone()),
                        _ => None
                    }
                });

                let session_index = doc
                    .descendants()
                    .find(|n| n.has_tag_name("AuthnStatement") || n.has_tag_name("saml:AuthnStatement"))
                    .and_then(|n| n.attribute("SessionIndex"))
                    .map(|s| s.to_string());

                log::info!("Parsed SAML response - NameID: {}, Email: {:?}, DisplayName: {:?}",
                    name_id, email, display_name);

                Ok(SamlResponse {
                    name_id,
                    email,
                    first_name,
                    last_name,
                    display_name,
                    attributes,
                    session_index,
                })
            }

            fn extract_attribute(&self, attributes: &HashMap<String, Vec<String>>, keys: &[&str]) -> Option<String> {
                for key in keys {
                    if let Some(values) = attributes.get(*key) {
                        if let Some(value) = values.first() {
                            if !value.trim().is_empty() {
                                return Some(value.clone());
                            }
                        }
                    }
                }
                None
            }

            pub fn generate_logout_request(&self, name_id: &str, session_index: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
                let request_id = format!("_{}", uuid::Uuid::new_v4().simple());
                let issue_instant = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");

                let session_index_xml = if let Some(index) = session_index {
                    format!("<samlp:SessionIndex>{}</samlp:SessionIndex>", index)
                } else {
                    String::new()
                };

                let logout_request_xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <samlp:LogoutRequest 
        xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
        xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
        ID="{}"
        Version="2.0"
        IssueInstant="{}"
        Destination="">
        <saml:Issuer>{}/saml/metadata</saml:Issuer>
        <saml:NameID Format="urn:oasis:names:tc:SAML:2.0:nameid-format:emailAddress">{}</saml:NameID>
        {}
    </samlp:LogoutRequest>"#, 
                        request_id,
                        issue_instant,
                        self.base_url,
                        name_id,
                        session_index_xml
                    );

                let encoded = general_purpose::STANDARD.encode(&logout_request_xml);
                Ok(encoded)
            }
        }

        pub async fn get_saml_config(pool: &Pool<Postgres>, institution_id: &str) -> Result<Option<SamlConfig>, ServerFnError> {
            let row = sqlx::query(
                "SELECT id, institution_name, entity_id, sso_url, slo_url, x509_cert, metadata_url,
                        active, created_at, updated_at, attribute_mapping, role_mapping, 
                        auto_provision, require_encrypted_assertions
                 FROM saml_configs WHERE institution_name = $1 AND active = true"
            )
            .bind(institution_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            match row {
                Some(row) => {
                    let attribute_mapping: serde_json::Value = row.get("attribute_mapping");
                    let role_mapping: serde_json::Value = row.get("role_mapping");

                    Ok(Some(SamlConfig {
                        id: row.get("id"),
                        institution_name: row.get("institution_name"),
                        entity_id: row.get("entity_id"),
                        sso_url: row.get("sso_url"),
                        slo_url: row.get("slo_url"),
                        x509_cert: row.get("x509_cert"),
                        metadata_url: row.get("metadata_url"),
                        active: row.get("active"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                        attribute_mapping: serde_json::from_value(attribute_mapping).unwrap_or_default(),
                        role_mapping: serde_json::from_value(role_mapping).unwrap_or_default(),
                        auto_provision: row.get("auto_provision"),
                        require_encrypted_assertions: row.get("require_encrypted_assertions"),
                    }))
                }
                None => Ok(None),
            }
        }

        // FIXED: Use manual query instead of query_as! to handle type conversions
        pub async fn get_saml_config_by_name(
            pool: &sqlx::PgPool,
            institution_name: &str,
        ) -> Result<Option<SamlConfig>, sqlx::Error> {
            let row = sqlx::query(
                "SELECT id, institution_name, entity_id, sso_url, slo_url, x509_cert, metadata_url,
                        active, created_at, updated_at, attribute_mapping, role_mapping, 
                        auto_provision, require_encrypted_assertions
                 FROM saml_configs WHERE institution_name = $1 AND active = true"
            )
            .bind(institution_name)
            .fetch_optional(pool)
            .await?;

            match row {
                Some(row) => {
                    // Manual conversion from time::OffsetDateTime to chrono::DateTime<Utc>
                    let created_at: time::OffsetDateTime = row.get("created_at");
                    let updated_at: time::OffsetDateTime = row.get("updated_at");

                    // Convert time::OffsetDateTime to chrono::DateTime<Utc>
                    let created_at_chrono = DateTime::<Utc>::from_timestamp(
                        created_at.unix_timestamp(),
                        created_at.nanosecond()
                    ).unwrap_or_else(|| Utc::now());

                    let updated_at_chrono = DateTime::<Utc>::from_timestamp(
                        updated_at.unix_timestamp(),
                        updated_at.nanosecond()
                    ).unwrap_or_else(|| Utc::now());

                    // Manual conversion for JSON fields
                    let attribute_mapping: serde_json::Value = row.get("attribute_mapping");
                    let role_mapping: serde_json::Value = row.get("role_mapping");

                    Ok(Some(SamlConfig {
                        id: row.get("id"),
                        institution_name: row.get("institution_name"),
                        entity_id: row.get("entity_id"),
                        sso_url: row.get("sso_url"),
                        slo_url: row.get("slo_url"),
                        x509_cert: row.get("x509_cert"),
                        metadata_url: row.get("metadata_url"),
                        active: row.get("active"),
                        created_at: created_at_chrono,
                        updated_at: updated_at_chrono,
                        attribute_mapping: serde_json::from_value(attribute_mapping).unwrap_or_default(),
                        role_mapping: serde_json::from_value(role_mapping).unwrap_or_default(),
                        auto_provision: row.get("auto_provision"),
                        require_encrypted_assertions: row.get("require_encrypted_assertions"),
                    }))
                }
                None => Ok(None),
            }
        }

        // Just-in-time user provisioning
        pub async fn provision_saml_user(
            pool: &Pool<Postgres>,
            saml_response: &SamlResponse,
            institution_id: &str,
        ) -> Result<SessionUser, ServerFnError> {
            // First check if user already exists with SAML mapping
            if let Some(existing_user) = get_user_by_saml_mapping(pool, institution_id, &saml_response.name_id).await? {
                // Update last login time
                update_saml_user_mapping_login(pool, existing_user.id, institution_id).await?;
                return Ok(existing_user);
            }

            // Check if user exists by email
            let email = saml_response.email.as_ref().unwrap_or(&saml_response.name_id);
            if let Some(existing_user) = user_database::get_user_by_email(pool, email).await? {
                // Link existing user to SAML
                link_user_to_saml(pool, existing_user.id, institution_id, &saml_response.name_id).await?;
                return Ok(existing_user.to_session_user());
            }

            // Create new user with SAML data
            let username = email.split('@').next().unwrap_or(&saml_response.name_id).to_string();

            // Generate a random password (user won't use it for SAML login)
            let temp_password = uuid::Uuid::new_v4().to_string();

            // Determine role from SAML attributes
            let role = determine_role_from_saml_attributes(&saml_response.attributes);

            // Create user with profile data from SAML
            let user = create_user_from_saml(pool, username, email.clone(), temp_password, role, saml_response).await?;

            // Store SAML association
            store_saml_user_association(pool, user.id, institution_id, &saml_response.name_id).await?;

            Ok(user)
        }

        async fn create_user_from_saml(
            pool: &Pool<Postgres>,
            username: String,
            email: String,
            password: String,
            role: UserRole,
            saml_response: &SamlResponse,
        ) -> Result<SessionUser, ServerFnError> {
            use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng}};

            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| ServerFnError::new(format!("Password hashing error: {}", e)))?
                .to_string();

            // Map UserRole to database enum values
            let role_str = match role {
                UserRole::Admin => "admin",
                UserRole::Teacher => "teacher",
                UserRole::User => "user",           // This maps to DB 'user'
                UserRole::Guest => "guest",         // This maps to DB 'guest'
                UserRole::SuperAdmin => "superadmin",
            };

            // Provide sensible defaults for missing fields
            let first_name = saml_response.first_name
                .as_ref()
                .filter(|name| !name.trim().is_empty())
                .cloned()
                .or_else(|| {
                    // Extract from email if no first name
                    email.split('@').next()
                        .map(|name| name.to_string())
                })
                .unwrap_or_else(|| "User".to_string());

            let last_name = saml_response.last_name
                .as_ref()
                .filter(|name| !name.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| "".to_string());

            let display_name = saml_response.display_name
                .as_ref()
                .filter(|name| !name.trim().is_empty())
                .cloned()
                .or_else(|| {
                    // Construct from first/last name
                    if !last_name.is_empty() {
                        Some(format!("{} {}", first_name, last_name).trim().to_string())
                    } else {
                        Some(first_name.clone())
                    }
                })
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| username.clone());

            log::info!("Creating SAML user: {} ({}) with role: {}", username, email, role_str);

            let row = sqlx::query(
                "INSERT INTO users (
                    username, 
                    email, 
                    password_hash, 
                    role, 
                    account_status, 
                    email_verified,
                    first_name, 
                    last_name, 
                    display_name, 
                    created_at, 
                    updated_at
                )
                VALUES ($1, $2, $3, $4::user_role_enum, 'active'::account_status_enum, true, $5, $6, $7, NOW(), NOW())
                RETURNING id, username, email, role, display_name, first_name, last_name"
            )
            .bind(&username)
            .bind(&email)
            .bind(&password_hash)
            .bind(role_str)
            .bind(&first_name)
            .bind(&last_name)
            .bind(&display_name)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                log::error!("Failed to create SAML user: {}", e);
                ServerFnError::new(format!("Failed to create user: {}", e))
            })?;

            Ok(SessionUser {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                role,
                display_name: Some(row.get("display_name")),
                first_name: Some(row.get("first_name")),
                last_name: Some(row.get("last_name")),
            })
        }

        fn determine_role_from_saml_attributes(attributes: &HashMap<String, Vec<String>>) -> UserRole {
            // Check for role attributes in common SAML claim formats
            let role_claims = [
                "http://schemas.microsoft.com/ws/2008/06/identity/claims/role",
                "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/role",
                "eduPersonAffiliation",
                "memberOf",
                "groups",
                "role",
                "roles",
            ];

            for claim in &role_claims {
                if let Some(roles) = attributes.get(*claim) {
                    for role in roles {
                        let role_lower = role.to_lowercase();

                        // Check for admin roles
                        if role_lower.contains("admin") ||
                           role_lower.contains("administrator") ||
                           role_lower.contains("superadmin") {
                            return UserRole::Admin;
                        }

                        // Check for teacher/instructor roles
                        if role_lower.contains("teacher") ||
                           role_lower.contains("instructor") ||
                           role_lower.contains("faculty") ||
                           role_lower.contains("staff") ||
                           role_lower.contains("educator") {
                            return UserRole::Teacher;
                        }

                        // Check for student roles
                        if role_lower.contains("student") ||
                           role_lower.contains("learner") {
                            return UserRole::User;  // Changed from UserRole::User to map to DB 'user'
                        }
                    }
                }
            }

            // Default to Guest role (safest default)
            UserRole::Guest
        }

        async fn get_user_by_saml_mapping(
            pool: &Pool<Postgres>,
            institution_id: &str,
            saml_name_id: &str,
        ) -> Result<Option<SessionUser>, ServerFnError> {
            let row = sqlx::query(
                "SELECT u.id, u.username, u.email, u.role, u.account_status, u.email_verified,
                        u.first_name, u.last_name, u.display_name, u.created_at
                 FROM users u
                 INNER JOIN saml_user_mappings sum ON u.id = sum.user_id
                 WHERE sum.institution_id = $1 AND sum.saml_name_id = $2 AND u.account_status = 'active'"
            )
            .bind(institution_id)
            .bind(saml_name_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            match row {
                Some(row) => {
                    let role: UserRole = row.get("role");

                    Ok(Some(SessionUser {
                        id: row.get("id"),
                        username: row.get("username"),
                        email: row.get("email"),
                        role,
                        display_name: row.get("display_name"),
                        first_name: row.get("first_name"),
                        last_name: row.get("last_name"),
                    }))
                }
                None => Ok(None),
            }
        }

        async fn link_user_to_saml(
            pool: &Pool<Postgres>,
            user_id: i64,
            institution_id: &str,
            saml_name_id: &str,
        ) -> Result<(), ServerFnError> {
            sqlx::query(
                "INSERT INTO saml_user_mappings (user_id, institution_id, saml_name_id, created_at, last_login)
                 VALUES ($1, $2, $3, NOW(), NOW())
                 ON CONFLICT (user_id, institution_id) DO UPDATE SET 
                 saml_name_id = $3, last_login = NOW()"
            )
            .bind(user_id)
            .bind(institution_id)
            .bind(saml_name_id)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to link user to SAML: {}", e)))?;

            Ok(())
        }

        async fn store_saml_user_association(
            pool: &Pool<Postgres>,
            user_id: i64,
            institution_id: &str,
            saml_name_id: &str,
        ) -> Result<(), ServerFnError> {
            sqlx::query(
                "INSERT INTO saml_user_mappings (user_id, institution_id, saml_name_id, created_at, last_login)
                 VALUES ($1, $2, $3, NOW(), NOW())"
            )
            .bind(user_id)
            .bind(institution_id)
            .bind(saml_name_id)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to store SAML association: {}", e)))?;

            Ok(())
        }

        async fn update_saml_user_mapping_login(
            pool: &Pool<Postgres>,
            user_id: i64,
            institution_id: &str,
        ) -> Result<(), ServerFnError> {
            sqlx::query(
                "UPDATE saml_user_mappings SET last_login = NOW()
                 WHERE user_id = $1 AND institution_id = $2"
            )
            .bind(user_id)
            .bind(institution_id)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to update login time: {}", e)))?;

            Ok(())
        }

        // Admin functions for SAML management
        pub async fn get_institution_users(
            pool: &Pool<Postgres>,
            institution_id: &str,
        ) -> Result<Vec<SessionUser>, ServerFnError> {
            let rows = sqlx::query(
                "SELECT u.id, u.username, u.email, u.role, u.account_status, u.email_verified,
                        u.first_name, u.last_name, u.display_name, u.created_at,
                        sum.last_login
                 FROM users u
                 INNER JOIN saml_user_mappings sum ON u.id = sum.user_id
                 WHERE sum.institution_id = $1 AND u.account_status = 'active'
                 ORDER BY sum.last_login DESC"
            )
            .bind(institution_id)
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let mut users = Vec::new();
            for row in rows {
                let role: UserRole = row.get("role"); // Direct read

                users.push(SessionUser {
                    id: row.get("id"),
                    username: row.get("username"),
                    email: row.get("email"),
                    role, // Use the directly read role
                    display_name: row.get("display_name"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                });
            }


            Ok(users)
        }

        // Update session to track auth provider
        pub async fn create_saml_session(
            pool: &Pool<Postgres>,
            user_id: i64,
            institution_id: &str
        ) -> Result<String, ServerFnError> {
            use rand::{distributions::Alphanumeric, Rng};

            let token: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(64)
                .map(char::from)
                .collect();

            sqlx::query(
                "INSERT INTO sessions (user_id, token, expires_at, created_at, auth_provider, institution_id)
                 VALUES ($1, $2, NOW() + INTERVAL '7 days', NOW(), 'saml', $3)"
            )
            .bind(user_id)
            .bind(&token)
            .bind(institution_id)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to create session: {}", e)))?;

            Ok(token)
        }

        pub async fn create_session_with_provider(
            pool: &Pool<Postgres>, user_id: i64,
            auth_provider: &str,
            institution_id: Option<&str>
        ) -> Result<String, sqlx::Error> {
            use rand::{distributions::Alphanumeric, Rng};

            let token: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(64)
                .map(char::from)
                .collect();

            sqlx::query(
                "INSERT INTO sessions (user_id, token, expires_at, created_at, auth_provider, institution_id)
                 VALUES ($1, $2, NOW() + INTERVAL '7 days', NOW(), $3, $4)"
            )
            .bind(user_id)
            .bind(&token)
            .bind(auth_provider)
            .bind(institution_id)
            .execute(pool)
            .await?;

            Ok(token)
        }

        pub async fn list_saml_configs(pool: &Pool<Postgres>) -> Result<Vec<SamlConfig>, ServerFnError> {
            let rows = sqlx::query(
                "SELECT id, institution_name, entity_id, sso_url, slo_url, x509_cert, metadata_url,
                        active, created_at, updated_at, attribute_mapping, role_mapping, 
                        auto_provision, require_encrypted_assertions
                 FROM saml_configs ORDER BY institution_name"
            )
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let mut configs = Vec::new();
            for row in rows {
                let attribute_mapping: serde_json::Value = row.get("attribute_mapping");
                let role_mapping: serde_json::Value = row.get("role_mapping");

                configs.push(SamlConfig {
                    id: row.get("id"),
                    institution_name: row.get("institution_name"),
                    entity_id: row.get("entity_id"),
                    sso_url: row.get("sso_url"),
                    slo_url: row.get("slo_url"),
                    x509_cert: row.get("x509_cert"),
                    metadata_url: row.get("metadata_url"),
                    active: row.get("active"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    attribute_mapping: serde_json::from_value(attribute_mapping).unwrap_or_default(),
                    role_mapping: serde_json::from_value(role_mapping).unwrap_or_default(),
                    auto_provision: row.get("auto_provision"),
                    require_encrypted_assertions: row.get("require_encrypted_assertions"),
                });
            }

            Ok(configs)
        }

        pub async fn create_saml_config(pool: &Pool<Postgres>, config: &SamlConfig) -> Result<(), ServerFnError> {
            let attribute_mapping_json = serde_json::to_value(&config.attribute_mapping)
                .map_err(|e| ServerFnError::new(format!("Failed to serialize attribute mapping: {}", e)))?;
            let role_mapping_json = serde_json::to_value(&config.role_mapping)
                .map_err(|e| ServerFnError::new(format!("Failed to serialize role mapping: {}", e)))?;

            sqlx::query(
                "INSERT INTO saml_configs (id, institution_name, entity_id, sso_url, slo_url, x509_cert,
                                          metadata_url, active, created_at, updated_at, attribute_mapping,
                                          role_mapping, auto_provision, require_encrypted_assertions)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"
            )
            .bind(config.id)
            .bind(&config.institution_name)
            .bind(&config.entity_id)
            .bind(&config.sso_url)
            .bind(&config.slo_url)
            .bind(&config.x509_cert)
            .bind(&config.metadata_url)
            .bind(config.active)
            .bind(config.created_at)
            .bind(config.updated_at)
            .bind(attribute_mapping_json)
            .bind(role_mapping_json)
            .bind(config.auto_provision)
            .bind(config.require_encrypted_assertions)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to create SAML config: {}", e)))?;

            Ok(())
        }
    }
}
