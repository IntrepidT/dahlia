use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountStatus {
    Pending,
    Active,
    Suspended,
    Deleted,
}
impl AccountStatus {
    pub fn to_string(&self) -> String {
        match self {
            AccountStatus::Pending => "pending".to_string(),
            AccountStatus::Active => "active".to_string(),
            AccountStatus::Suspended => "suspended".to_string(),
            AccountStatus::Deleted => "deleted".to_string(),
        }
    }

    pub fn from_str(status: &str) -> Self {
        match status {
            "pending" => AccountStatus::Pending,
            "active" => AccountStatus::Active,
            "suspended" => AccountStatus::Suspended,
            "deleted" => AccountStatus::Deleted,
            _ => AccountStatus::Pending,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserJwt {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
}
impl UserJwt {
    pub fn new(
        id: i64,
        username: String,
        email: String,
        password_hash: String,
        role: String,
    ) -> Self {
        UserJwt {
            id,
            username,
            email,
            password_hash,
            role,
        }
    }
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn is_teacher(&self) -> bool {
        self.role == "teacher" || self.role == "admin"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing, default)]
    pub password_hash: String,
    pub role: String,
    pub password_salt: Option<String>,
    pub account_status: AccountStatus,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub phone_verified: bool,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn is_teacher(&self) -> bool {
        self.role == "teacher" || self.role == "admin"
    }
}
