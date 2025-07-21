use crate::app::models::question::QuestionType;
use leptos::*;
use std::collections::HashMap;

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

#[derive(Clone)]
pub struct FlashCardState {
    pub responses: Signal<HashMap<i32, QuestionResponse>>,
    pub current_card_index: Signal<usize>,
    pub is_submitted: Signal<bool>,
    pub selected_student_id: Signal<Option<i32>>,
}

pub fn use_flash_card_state() -> (
    FlashCardState,
    WriteSignal<HashMap<i32, QuestionResponse>>,
    WriteSignal<usize>,
    WriteSignal<bool>,
    WriteSignal<Option<i32>>,
) {
    let (responses, set_responses) = create_signal(HashMap::<i32, QuestionResponse>::new());
    let (current_card_index, set_current_card_index) = create_signal(0);
    let (is_submitted, set_is_submitted) = create_signal(false);
    let (selected_student_id, set_selected_student_id) = create_signal(None::<i32>);

    let state = FlashCardState {
        responses: responses.into(),
        current_card_index: current_card_index.into(),
        is_submitted: is_submitted.into(),
        selected_student_id: selected_student_id.into(),
    };

    (state, set_responses, set_current_card_index, set_is_submitted, set_selected_student_id)
}
