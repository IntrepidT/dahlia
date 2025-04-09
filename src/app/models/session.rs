use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSession {
    pub session_id: String,
    pub test_id: String,
    pub student_id: i32,
    pub evaluator_id: String,
    pub current_card_index: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestSessionRequest {
    pub test_id: String,
    pub student_id: i32,
    pub evaluator_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTestSessionRequest {
    pub session_id: String,
    pub current_card_index: Option<i32>,
    pub is_active: Option<bool>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    NavigateToCard(usize),
    UpdateAnswer { question_id: i32, answer: String },
    UpdateComment { question_id: i32, comment: String },
    CompleteSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub event: SessionEvent,
    pub timestamp: DateTime<Utc>,
    pub sender_id: String,
}
