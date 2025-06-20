use crate::app::models::user::UserRole;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthProvider {
    Local,
    Saml(String), // Institution ID
    Google,
    Microsoft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String, // User ID
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub auth_provider: AuthProvider,
    pub session_id: String,
    pub institution_id: Option<String>,
    pub iat: i64,    // Issued at
    pub exp: i64,    // Expiration
    pub iss: String, // Issuer
    pub aud: String, // Audience
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConfig {
    pub id: Uuid,
    pub institution_name: String,
    pub entity_id: String,
    pub sso_url: String,
    pub slo_url: Option<String>,
    pub x509_cert: String,
    pub metadata_url: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub attribute_mapping: HashMap<String, String>,
    pub role_mapping: HashMap<String, String>,
    pub auto_provision: bool,
    pub require_encrypted_assertions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlResponse {
    pub name_id: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub attributes: HashMap<String, Vec<String>>,
    pub session_index: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub id: String,
    pub user_id: i64,
    pub auth_provider: AuthProvider,
    pub institution_id: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountStatus {
    Pending,
    Active,
    Suspended,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserJwt {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub account_status: AccountStatus,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserJwt,
}

// For registration
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}
