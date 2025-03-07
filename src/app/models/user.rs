use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing, default)]
    pub password_hash: String,
    pub role: String,
}

impl User {
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        User {
            id: 0, // Will be set by the database
            username,
            email,
            password_hash,
            role: "user".to_string(), // Default role
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn is_teacher(&self) -> bool {
        self.role == "teacher" || self.role == "admin"
    }
}
