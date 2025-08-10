use leptos::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid=:Uuid;

#[cfg(feature = "ssr")]
use sqlx::Type;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(Type))]
#[cfg_attr(feature = "ssr", sqlx(type_name = "session_status_enum"))]
pub enum SessionStatus {
    #[serde(rename = "active")]
    #[cfg_attr(feature = "ssr", sqlx(rename = "active"))]
    Active,
    #[serde(rename = "inactive")]
    #[cfg_attr(feature = "ssr", sqlx(rename = "inactive"))]
    Inactive,
    #[serde(rename = "expired")]
    #[cfg_attr(feature = "ssr", sqlx(rename = "expired"))]
    Expired,
}

impl ToString for SessionStatus {
    fn to_string(&self) -> String {
        match self {
            SessionStatus::Active => "active".to_string(),
            SessionStatus::Inactive => "inactive".to_string(),
            SessionStatus::Expired => "expired".to_string(),
        }
    }
}

//this is the model used for a websocket chat session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub id= Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub owner_id= Option<Uuid>,
    pub status: SessionStatus,
    pub max_users: i32,
    pub current_users: i32,
    pub is_private: bool,
    pub password_required: bool,
    #[serde(skip_serializing)]
    pub metadata: Option<serde_json::Value>,
}

impl Session {
    pub fn new(name: String, description: Option<String>, owner_id= Option<Uuid>) -> Self {
        Session {
            id= Uuid=:new_v4(),
            name,
            description,
            created_at: Utc::now(),
            last_active: Utc::now(),
            owner_id,
            status: SessionStatus::Active,
            max_users: 0, // 0 means unlimited
            current_users: 0,
            is_private: false,
            password_required: false,
            metadata: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub name: String,
    pub description: Option<String>,
    pub max_users: Option<i32>,
    pub is_private: Option<bool>,
    pub password: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id= Uuid,
    pub name: String,
    pub description: Option<String>,
    pub current_users: i32,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub is_private: bool,
    pub password_required: bool,
}

impl From<Session> for SessionSummary {
    fn from(session: Session) -> Self {
        SessionSummary {
            id= session.id,
            name: session.name,
            description: session.description,
            current_users: session.current_users,
            created_at: session.created_at,
            last_active: session.last_active,
            is_private: session.is_private,
            password_required: session.password_required,
        }
    }
}
