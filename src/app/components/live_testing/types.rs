use chrono::{DateTime, Utc};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone)]
pub struct QuestionResponse {
    pub answer: String,
    pub comment: String,
    pub selected_options: Option<Vec<String>>,
}

impl QuestionResponse {
    pub fn new() -> Self {
        Self {
            answer: String::new(),
            comment: String::new(),
            selected_options: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Teacher,
    Student,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ConnectedStudent {
    pub student_id: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct TestSessionState {
    pub room_id: Option<Uuid>,
    pub role: Role,
    pub connected_students: Vec<ConnectedStudent>,
    pub connection_status: ConnectionStatus,
    pub error_message: Option<String>,
    pub current_card_index: usize,
    pub responses: std::collections::HashMap<i32, QuestionResponse>,
    pub selected_student_id: Option<i32>,
    pub is_test_active: bool,
    pub is_submitted: bool,
    pub remaining_time: Option<i32>,
}

impl Default for TestSessionState {
    fn default() -> Self {
        Self {
            room_id: None,
            role: Role::Unknown,
            connected_students: Vec::new(),
            connection_status: ConnectionStatus::Disconnected,
            error_message: None,
            current_card_index: 0,
            responses: std::collections::HashMap::new(),
            selected_student_id: None,
            is_test_active: false,
            is_submitted: false,
            remaining_time: None,
        }
    }
}
