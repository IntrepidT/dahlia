use leptos::prelude::*;
use crate::app::models::question::{Question, QuestionType, WeightedOption};
use crate::app::models::score::{QuestionResponse, CreateScoreRequest};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScoringEngine;

impl ScoringEngine {
    /// Calculate score for a single question based on student's answer
    pub fn calculate_question_score(
        question: &Question,
        student_answer: &str,
        comment: &str,
    ) -> Result<QuestionResponse, String> {
        match &question.question_type {
            QuestionType::WeightedMultipleChoice => {
                Self::score_weighted_multiple_choice(question, student_answer, comment)
            }
            QuestionType::MultipleChoice => {
                Self::score_traditional_multiple_choice(question, student_answer, comment)
            }
            QuestionType::TrueFalse => {
                Self::score_true_false(question, student_answer, comment)
            }
            _ => Err(format!(
                "Question type {:?} is not supported for automated scoring",
                question.question_type
            )),
        }
    }

    /// Score a weighted multiple choice question
    fn score_weighted_multiple_choice(
        question: &Question,
        student_answer: &str,
        comment: &str,
    ) -> Result<QuestionResponse, String> {
        if let Some(weighted_options) = &question.weighted_options {
            // Find the selected option
            let selected_option = weighted_options
                .iter()
                .find(|opt| opt.text.trim().eq_ignore_ascii_case(student_answer.trim()));

            match selected_option {
                Some(option) => {
                    let max_points = weighted_options
                        .iter()
                        .map(|opt| opt.points)
                        .max()
                        .unwrap_or(0);

                    Ok(QuestionResponse::new(
                        question.qnumber,
                        student_answer.to_string(),
                        option.points,
                        max_points,
                        comment.to_string(),
                        option.is_correct,
                    ))
                }
                None => {
                    // Student provided an answer not in the options - give 0 points
                    let max_points = weighted_options
                        .iter()
                        .map(|opt| opt.points)
                        .max()
                        .unwrap_or(0);

                    Ok(QuestionResponse::new(
                        question.qnumber,
                        student_answer.to_string(),
                        0,
                        max_points,
                        format!("{} (Answer not found in options)", comment),
                        false,
                    ))
                }
            }
        } else {
            Err("Weighted multiple choice question missing weighted options".to_string())
        }
    }

    /// Score a traditional multiple choice question
    fn score_traditional_multiple_choice(
        question: &Question,
        student_answer: &str,
        comment: &str,
    ) -> Result<QuestionResponse, String> {
        let is_correct = question.correct_answer.eq_ignore_ascii_case(student_answer);
        let points_earned = if is_correct { question.point_value } else { 0 };

        Ok(QuestionResponse::new(
            question.qnumber,
            student_answer.to_string(),
            points_earned,
            question.point_value,
            comment.to_string(),
            is_correct,
        ))
    }

    /// Score a true/false question
    fn score_true_false(
        question: &Question,
        student_answer: &str,
        comment: &str,
    ) -> Result<QuestionResponse, String> {
        let is_correct = question.correct_answer.eq_ignore_ascii_case(student_answer);
        let points_earned = if is_correct { question.point_value } else { 0 };

        Ok(QuestionResponse::new(
            question.qnumber,
            student_answer.to_string(),
            points_earned,
            question.point_value,
            comment.to_string(),
            is_correct,
        ))
    }

    /// Score an entire test given questions and student responses
    pub fn score_test(
        questions: &[Question],
        student_responses: &HashMap<i32, (String, String)>, // qnumber -> (answer, comment)
        student_id= i32,
        test_id= String,
        test_variant: i32,
        evaluator: String,
    ) -> Result<CreateScoreRequest, String> {
        let mut question_responses = Vec::new();
        let mut errors = Vec::new();

        for question in questions {
            let (student_answer, comment) = student_responses
                .get(&question.qnumber)
                .cloned()
                .unwrap_or_else(|| ("".to_string(), "No answer provided".to_string()));

            match Self::calculate_question_score(question, &student_answer, &comment) {
                Ok(response) => question_responses.push(response),
                Err(e) => errors.push(format!("Question {}: {}", question.qnumber, e)),
            }
        }

        if !errors.is_empty() {
            return Err(format!("Scoring errors: {}", errors.join("; ")));
        }

        // Sort responses by question number to ensure consistent ordering
        question_responses.sort_by_key(|r| r.qnumber);

        Ok(CreateScoreRequest::new_with_responses(
            student_id,
            test_id,
            question_responses,
            test_variant,
            evaluator,
        ))
    }

    /// Get detailed analytics for a scored test
    pub fn analyze_test_performance(
        questions: &[Question],
        question_responses: &[QuestionResponse],
    ) -> TestPerformanceAnalysis {
        let mut analysis = TestPerformanceAnalysis::new();

        for response in question_responses {
            analysis.total_questions += 1;
            
            if !response.student_answer.trim().is_empty() {
                analysis.questions_answered += 1;
            }
            
            if response.is_correct {
                analysis.questions_correct += 1;
            }
            
            analysis.total_points_earned += response.points_earned;
            analysis.total_points_possible += response.max_possible_points;

            // Analyze by question type
            if let Some(question) = questions.iter().find(|q| q.qnumber == response.qnumber) {
                let type_stats = analysis.question_type_breakdown
                    .entry(question.question_type.clone())
                    .or_insert_with(QuestionTypeStats::new);
                
                type_stats.total += 1;
                type_stats.points_earned += response.points_earned;
                type_stats.points_possible += response.max_possible_points;
                
                if response.is_correct {
                    type_stats.correct += 1;
                }
            }

            // Track individual question performance
            analysis.question_details.push(QuestionPerformance {
                qnumber: response.qnumber,
                points_earned: response.points_earned,
                max_possible_points: response.max_possible_points,
                percentage: if response.max_possible_points > 0 {
                    (response.points_earned as f64 / response.max_possible_points as f64) * 100.0
                } else {
                    0.0
                },
                is_correct: response.is_correct,
                student_answer: response.student_answer.clone(),
                comment: response.comment.clone(),
            });
        }

        // Calculate overall percentage
        analysis.overall_percentage = if analysis.total_points_possible > 0 {
            (analysis.total_points_earned as f64 / analysis.total_points_possible as f64) * 100.0
        } else {
            0.0
        };

        analysis
    }
}

#[derive(Debug, Clone)]
pub struct TestPerformanceAnalysis {
    pub total_questions: usize,
    pub questions_answered: usize,
    pub questions_correct: usize,
    pub total_points_earned: i32,
    pub total_points_possible: i32,
    pub overall_percentage: f64,
    pub question_type_breakdown: HashMap<QuestionType, QuestionTypeStats>,
    pub question_details: Vec<QuestionPerformance>,
}

impl TestPerformanceAnalysis {
    fn new() -> Self {
        Self {
            total_questions: 0,
            questions_answered: 0,
            questions_correct: 0,
            total_points_earned: 0,
            total_points_possible: 0,
            overall_percentage: 0.0,
            question_type_breakdown: HashMap::new(),
            question_details: Vec::new(),
        }
    }

    /// Get the grade based on percentage (you can customize this)
    pub fn get_letter_grade(&self) -> String {
        match self.overall_percentage {
            p if p >= 97.0 => "A+".to_string(),
            p if p >= 93.0 => "A".to_string(),
            p if p >= 90.0 => "A-".to_string(),
            p if p >= 87.0 => "B+".to_string(),
            p if p >= 83.0 => "B".to_string(),
            p if p >= 80.0 => "B-".to_string(),
            p if p >= 77.0 => "C+".to_string(),
            p if p >= 73.0 => "C".to_string(),
            p if p >= 70.0 => "C-".to_string(),
            p if p >= 67.0 => "D+".to_string(),
            p if p >= 65.0 => "D".to_string(),
            _ => "F".to_string(),
        }
    }

    /// Get strengths and weaknesses based on question type performance
    pub fn get_performance_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();

        for (question_type, stats) in &self.question_type_breakdown {
            let percentage = if stats.points_possible > 0 {
                (stats.points_earned as f64 / stats.points_possible as f64) * 100.0
            } else {
                0.0
            };

            let performance_level = match percentage {
                p if p >= 90.0 => "excellent",
                p if p >= 80.0 => "good",
                p if p >= 70.0 => "fair", 
                _ => "needs improvement",
            };

            insights.push(format!(
                "{:?} questions: {:.1}% ({} out of {}) - {}",
                question_type, percentage, stats.correct, stats.total, performance_level
            ));
        }

        insights
    }
}

#[derive(Debug, Clone)]
pub struct QuestionTypeStats {
    pub total: usize,
    pub correct: usize,
    pub points_earned: i32,
    pub points_possible: i32,
}

impl QuestionTypeStats {
    fn new() -> Self {
        Self {
            total: 0,
            correct: 0,
            points_earned: 0,
            points_possible: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuestionPerformance {
    pub qnumber: i32,
    pub points_earned: i32,
    pub max_possible_points: i32,
    pub percentage: f64,
    pub is_correct: bool,
    pub student_answer: String,
    pub comment: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::models::question::{Question, QuestionType, WeightedOption};

    #[test]
    fn test_weighted_multiple_choice_scoring() {
        let weighted_options = vec![
            WeightedOption::new("Excellent approach".to_string(), 10, true),
            WeightedOption::new("Good approach".to_string(), 7, false),
            WeightedOption::new("Fair approach".to_string(), 4, false),
            WeightedOption::new("Poor approach".to_string(), 1, false),
        ];

        let question = Question::new_weighted(
            "What's the best way to solve this problem?".to_string(),
            weighted_options,
            1,
            "test-id".to_string(),
        );

        // Test scoring different answers
        let response1 = ScoringEngine::calculate_question_score(
            &question,
            "Excellent approach",
            "Great job!",
        ).unwrap();
        
        assert_eq!(response1.points_earned, 10);
        assert_eq!(response1.max_possible_points, 10);
        assert!(response1.is_correct);

        let response2 = ScoringEngine::calculate_question_score(
            &question,
            "Good approach", 
            "Nice work",
        ).unwrap();
        
        assert_eq!(response2.points_earned, 7);
        assert_eq!(response2.max_possible_points, 10);
        assert!(!response2.is_correct);
    }

    #[test]
    fn test_traditional_multiple_choice_compatibility() {
        let question = Question::new(
            "What is 2 + 2?".to_string(),
            5,
            QuestionType::MultipleChoice,
            vec!["3".to_string(), "4".to_string(), "5".to_string()],
            "4".to_string(),
            1,
            "test-id".to_string(),
        );

        let correct_response = ScoringEngine::calculate_question_score(
            &question,
            "4",
            "Correct!",
        ).unwrap();

        assert_eq!(correct_response.points_earned, 5);
        assert_eq!(correct_response.max_possible_points, 5);
        assert!(correct_response.is_correct);

        let incorrect_response = ScoringEngine::calculate_question_score(
            &question,
            "3",
            "Try again",
        ).unwrap();

        assert_eq!(incorrect_response.points_earned, 0);
        assert_eq!(incorrect_response.max_possible_points, 5);
        assert!(!incorrect_response.is_correct);
    }
}
