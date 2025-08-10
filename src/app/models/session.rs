use leptos::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid=:Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id= Uuid,
    pub session_code: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinSessionRequest {
    pub session_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionJoinResponse {
    pub session_id= Uuid,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeacherSession {
    pub id= Uuid,
    pub session_code: String,
    pub title: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
