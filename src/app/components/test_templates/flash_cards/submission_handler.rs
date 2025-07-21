use super::flash_card_state::QuestionResponse;
use crate::app::models::question::{Question, QuestionType};
use crate::app::models::score::CreateScoreRequest;
use crate::app::server_functions::scores::add_score;
use leptos::*;
use log;
use std::collections::HashMap;

pub fn create_submission_action(
    test_id: Signal<String>,
    questions: Signal<Option<Vec<Question>>>,
    evaluator_id: Signal<String>,
) -> Action<(HashMap<i32, QuestionResponse>, Option<i32>), Result<(), leptos::ServerFnError>> {
    create_action(
        move |(responses, student_id): &(HashMap<i32, QuestionResponse>, Option<i32>)| {
            let current_responses = responses.clone();
            let current_test_id = test_id.get();
            let student_id = student_id.unwrap_or(0);
            let evaluator = evaluator_id.get();
            let test_variant = 1;

            async move {
                let mut test_scores = Vec::new();
                let mut comments = Vec::new();

                if let Some(questions_vec) = questions.get() {
                    let mut sorted_questions = questions_vec.clone();
                    sorted_questions.sort_by_key(|q| q.qnumber);

                    for question in sorted_questions {
                        if let Some(response) = current_responses.get(&question.qnumber) {
                            let score = match question.question_type {
                                QuestionType::WeightedMultipleChoice => {
                                    if let Some(ref selected_opts) = response.selected_options {
                                        question.calculate_weighted_score(selected_opts)
                                    } else {
                                        0
                                    }
                                }
                                _ => {
                                    if response.answer == question.correct_answer {
                                        question.point_value
                                    } else {
                                        0
                                    }
                                }
                            };

                            test_scores.push(score);
                            comments.push(response.comment.clone());
                        } else {
                            test_scores.push(0);
                            comments.push(String::new());
                        }
                    }
                }

                let score_request = CreateScoreRequest {
                    student_id,
                    test_id: current_test_id,
                    test_scores,
                    comments,
                    test_variant,
                    evaluator,
                };

                match add_score(score_request).await {
                    Ok(score) => {
                        log::info!(
                            "Successfully submitted score for student {}",
                            score.student_id
                        );
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("Failed to submit score: {}", e);
                        Err(e)
                    }
                }
            }
        },
    )
}
