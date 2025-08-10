use crate::app::models::assessment::Assessment;
use crate::app::models::score::Score;
use crate::app::models::student::Student;
use crate::app::models::test::Test;
use crate::app::server_functions::teachers::get_teachers;
use crate::app::server_functions::{
    assessments::get_assessments, scores::get_student_scores, students::get_student,
    tests::get_tests,
};
use chrono::prelude::*;
use futures::join;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Debug};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Progress {
    Completed,
    Ongoing,
    NotStarted,
}

impl fmt::Display for Progress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Progress::Completed => "Completed".to_string(),
                Progress::Ongoing => "In Progress".to_string(),
                Progress::NotStarted => "Not Started".to_string(),
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct TestHistoryEntry {
    pub test_id: String,
    pub test_name: String,
    pub score: i32,
    pub total_possible: i32,
    pub date_administered: DateTime<Utc>,
    pub performance_class: String,
    pub evaluator: String,
    pub attempt: i32,
}

// New structures to represent pre-processed data for efficient rendering
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct StudentResultsSummary {
    pub student: Student,
    pub assessment_summaries: Vec<AssessmentSummary>,
    pub test_summaries: Vec<TestDetail>,
    pub test_history: Vec<TestHistoryEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AssessmentSummary {
    pub assessment_id: String,
    pub assessment_name: String,
    pub subject: String,
    pub total_possible: Option<i32>,
    pub current_score: i32,
    pub grade_level: Option<String>,
    pub test_details: Vec<TestDetail>, // Now contains ALL attempts, not just highest
    pub distribution_data: Vec<(String, i32)>,
    pub assessment_rating: String,
    pub progress: Progress,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct TestDetail {
    pub test_id: String,
    pub test_name: String,
    pub score: i32,
    pub total_possible: i32,
    pub test_area: String,
    pub date_administered: DateTime<Utc>,
    pub performance_class: String,
    pub attempt: i32,
    pub test_variant: i32,
}

#[cfg(feature = "ssr")]
pub async fn get_student_results(student_id: i32) -> Result<StudentResultsSummary, String> {
    // Parallel data fetching instead of sequential - major performance improvement
    let (tests_result, assessments_result, scores_result, student_result, teachers_result) = join!(
        get_tests(),
        get_assessments(),
        get_student_scores(student_id),
        get_student(student_id),
        get_teachers(),
    );

    // Handle results with proper error propagation
    let tests = tests_result.map_err(|e| e.to_string())?;
    let assessments = assessments_result.map_err(|e| e.to_string())?;
    let scores = scores_result.map_err(|e| e.to_string())?;
    let student = student_result.map_err(|e| e.to_string())?;
    let teachers = teachers_result.map_err(|e| e.to_string())?;

    // Create efficient lookup maps - O(1) access instead of O(n) searches
    let test_lookup: HashMap<String, &Test> = tests
        .iter()
        .map(|test| (test.test_id.clone(), test))
        .collect();

    let assessment_lookup: HashMap<String, &Assessment> = assessments
        .iter()
        .map(|assessment| (assessment.id.to_string(), assessment))
        .collect();

    let teacher_lookup: HashMap<String, String> = teachers
        .iter()
        .map(|teacher| {
            let full_name = format!("{} {}", teacher.firstname, teacher.lastname);
            let teacher_id = teacher.id.to_string();
            log::info!("Teacher lookup entry: '{}' -> '{}'", teacher_id, full_name);
            (teacher_id, full_name)
        })
        .collect();

    log::info!(
        "Created teacher lookup with {} entries",
        teacher_lookup.len()
    );
    if teacher_lookup.is_empty() {
        log::warn!("Teacher lookup is empty! No teachers found.");
    }

    // Process test history efficiently without dataframes
    let test_history = build_test_history_fixed(&scores, &test_lookup, &teacher_lookup);

    // MODIFIED: Process ALL scores instead of just highest scores
    let all_test_details = build_all_test_details(&scores, &test_lookup, &teacher_lookup);

    // Find highest scores for the test_summaries field (backwards compatibility)
    let highest_scores = find_highest_scores(&scores);
    let highest_test_details =
        build_test_details_fixed(&highest_scores, &test_lookup, &teacher_lookup);

    // Group by assessment and create summaries with ALL attempts
    let assessment_summaries =
        build_assessment_summaries_with_all_attempts(&all_test_details, &assessment_lookup);

    Ok(StudentResultsSummary {
        student,
        assessment_summaries,
        test_summaries: highest_test_details, // Keep this as highest scores for backwards compatibility
        test_history,
    })
}

// NEW: Build test details for ALL scores, not just highest
#[cfg(feature = "ssr")]
fn build_all_test_details(
    scores: &[Score],
    test_lookup: &HashMap<String, &Test>,
    teacher_lookup: &HashMap<String, String>,
) -> Vec<TestDetail> {
    scores
        .iter()
        .filter_map(|score| {
            test_lookup.get(&score.test_id).map(|test| {
                let score_total = score.get_total();
                let evaluator_name = resolve_evaluator_name(&score.evaluator, teacher_lookup);

                TestDetail {
                    test_id: score.test_id.clone(),
                    test_name: test.name.clone(),
                    score: score_total,
                    total_possible: test.score,
                    test_area: test.testarea.to_string(),
                    date_administered: score.date_administered,
                    performance_class: determine_performance_class_fast(test, score_total),
                    attempt: score.attempt,
                    test_variant: score.test_variant,
                }
            })
        })
        .collect()
}

// MODIFIED: Assessment summary builder that includes ALL attempts
#[cfg(feature = "ssr")]
fn build_assessment_summaries_with_all_attempts(
    all_test_details: &[TestDetail],
    assessment_lookup: &HashMap<String, &Assessment>,
) -> Vec<AssessmentSummary> {
    // Group ALL test attempts by assessment efficiently
    let mut assessment_tests: HashMap<String, Vec<TestDetail>> = HashMap::new();

    for test_detail in all_test_details {
        for (assessment_id, assessment) in assessment_lookup {
            if assessment
                .tests
                .iter()
                .any(|uuid| uuid.to_string() == test_detail.test_id)
            {
                assessment_tests
                    .entry(assessment_id.clone())
                    .or_default()
                    .push(test_detail.clone());
            }
        }
    }

    // Create summaries using iterator patterns - very efficient
    assessment_tests
        .into_iter()
        .filter_map(|(assessment_id, mut test_details)| {
            assessment_lookup.get(&assessment_id).map(|assessment| {
                // Sort test details by date (most recent first) and then by attempt
                test_details.sort_unstable_by(|a, b| {
                    b.date_administered
                        .cmp(&a.date_administered)
                        .then_with(|| b.attempt.cmp(&a.attempt))
                });

                // For current_score calculation, use the highest score for each unique test
                let highest_scores_per_test = find_highest_scores_per_test(&test_details);
                let current_score: i32 = highest_scores_per_test.values().map(|td| td.score).sum();
                let total_possible = assessment.composite_score;

                AssessmentSummary {
                    assessment_id: assessment_id.clone(),
                    assessment_name: assessment.name.clone(),
                    subject: assessment
                        .subject
                        .map_or("Unknown".to_string(), |s| s.to_string()),
                    total_possible,
                    current_score,
                    grade_level: assessment.grade.as_ref().map(|g| g.to_string()),
                    test_details: test_details.clone(),
                    distribution_data: calculate_distribution_fast(&test_details),
                    assessment_rating: determine_assessment_rating_fast(
                        assessment,
                        current_score,
                        total_possible.unwrap_or(0),
                    ),
                    progress: calculate_progress_fast_all_attempts(assessment, &test_details),
                }
            })
        })
        .collect()
}

// NEW: Helper function to find highest scores per test from a list of test details
#[cfg(feature = "ssr")]
fn find_highest_scores_per_test(test_details: &[TestDetail]) -> HashMap<String, &TestDetail> {
    let mut highest_per_test: HashMap<String, &TestDetail> = HashMap::new();

    for test_detail in test_details {
        match highest_per_test.get(&test_detail.test_id) {
            Some(existing) if existing.score >= test_detail.score => {
                // Keep existing highest score
            }
            _ => {
                // Update with new highest score
                highest_per_test.insert(test_detail.test_id.clone(), test_detail);
            }
        }
    }

    highest_per_test
}

// MODIFIED: Progress calculation that works with all attempts
#[cfg(feature = "ssr")]
fn calculate_progress_fast_all_attempts(
    assessment: &Assessment,
    test_details: &[TestDetail],
) -> Progress {
    let assessment_test_ids: std::collections::HashSet<String> = assessment
        .tests
        .iter()
        .map(|uuid| uuid.to_string())
        .collect();

    // Get unique test IDs that have been attempted
    let attempted_test_ids: std::collections::HashSet<String> =
        test_details.iter().map(|td| td.test_id.clone()).collect();

    let attempted_count = attempted_test_ids.len();
    let total_count = assessment_test_ids.len();

    match attempted_count {
        0 => Progress::NotStarted,
        n if n == total_count => {
            // Verify all required tests have been attempted
            if assessment_test_ids == attempted_test_ids {
                Progress::Completed
            } else {
                Progress::Ongoing
            }
        }
        _ => Progress::Ongoing,
    }
}

// Optimized test history builder - no dataframes needed
#[cfg(feature = "ssr")]
fn build_test_history_fixed(
    scores: &[Score],
    test_lookup: &HashMap<String, &Test>,
    teacher_lookup: &HashMap<String, String>,
) -> Vec<TestHistoryEntry> {
    let mut history: Vec<TestHistoryEntry> = scores
        .iter()
        .filter_map(|score| {
            test_lookup.get(&score.test_id).map(|test| {
                let score_total = score.get_total();
                let evaluator_name = resolve_evaluator_name(&score.evaluator, teacher_lookup);

                TestHistoryEntry {
                    test_id: score.test_id.clone(),
                    test_name: test.name.clone(),
                    score: score_total,
                    total_possible: test.score,
                    date_administered: score.date_administered,
                    performance_class: determine_performance_class_fast(test, score_total),
                    evaluator: evaluator_name,
                    attempt: score.attempt,
                }
            })
        })
        .collect();

    // Sort by date (much faster than dataframe sorting)
    history.sort_unstable_by(|a, b| b.date_administered.cmp(&a.date_administered));
    history
}

// Optimized highest score finding - O(n) single pass
#[cfg(feature = "ssr")]
fn find_highest_scores(scores: &[Score]) -> HashMap<String, &Score> {
    let mut highest_scores: HashMap<String, &Score> = HashMap::new();

    for score in scores {
        let score_total = score.get_total();
        match highest_scores.get(&score.test_id) {
            Some(existing_score) if existing_score.get_total() >= score_total => {
                // Keep existing highest score
            }
            _ => {
                // Update with new highest score
                highest_scores.insert(score.test_id.clone(), score);
            }
        }
    }

    highest_scores
}

// Native test details processing - no dataframe overhead
#[cfg(feature = "ssr")]
fn build_test_details_fixed(
    highest_scores: &HashMap<String, &Score>,
    test_lookup: &HashMap<String, &Test>,
    teacher_lookup: &HashMap<String, String>,
) -> Vec<TestDetail> {
    highest_scores
        .iter()
        .filter_map(|(test_id, score)| {
            test_lookup.get(test_id).map(|test| {
                let score_total = score.get_total();
                let evaluator_name = resolve_evaluator_name(&score.evaluator, teacher_lookup);

                TestDetail {
                    test_id: score.test_id.clone(),
                    test_name: test.name.clone(),
                    score: score_total,
                    total_possible: test.score,
                    test_area: test.testarea.to_string(),
                    date_administered: score.date_administered,
                    performance_class: determine_performance_class_fast(test, score_total),
                    attempt: score.attempt,
                    test_variant: score.test_variant,
                }
            })
        })
        .collect()
}

// DEPRECATED: Keep for backwards compatibility but rename to indicate it's old
#[cfg(feature = "ssr")]
fn build_assessment_summaries(
    test_details: &[TestDetail],
    assessment_lookup: &HashMap<String, &Assessment>,
) -> Vec<AssessmentSummary> {
    // This now just calls the new function for backwards compatibility
    build_assessment_summaries_with_all_attempts(test_details, assessment_lookup)
}

// Optimized performance classification - avoid repeated benchmark iteration
#[cfg(feature = "ssr")]
fn determine_performance_class_fast(test: &Test, score: i32) -> String {
    if let Some(benchmark_categories) = &test.benchmark_categories {
        let percentage = (score as f32 / test.score as f32) * 100.0;

        // Use early return pattern for better performance
        for benchmark in benchmark_categories {
            if percentage >= benchmark.min as f32 && percentage <= benchmark.max as f32 {
                return benchmark.label.clone();
            }
        }
    }
    "Not Rated".to_string()
}

// Native distribution calculation - much faster than dataframe groupby
#[cfg(feature = "ssr")]
fn calculate_distribution_fast(test_details: &[TestDetail]) -> Vec<(String, i32)> {
    let mut counts: HashMap<String, i32> = HashMap::new();

    for test_detail in test_details {
        *counts
            .entry(test_detail.performance_class.clone())
            .or_insert(0) += 1;
    }

    counts.into_iter().collect()
}

// Fast assessment rating - avoid repeated benchmark checks
#[cfg(feature = "ssr")]
fn determine_assessment_rating_fast(
    assessment: &Assessment,
    score: i32,
    total_possible: i32,
) -> String {
    if total_possible == 0 {
        return "Not Rated".to_string();
    }

    let percentage = (score as f32 / total_possible as f32) * 100.0;

    // Check risk benchmarks first (early return pattern)
    if let Some(risk_benchmarks) = &assessment.risk_benchmarks {
        for benchmark in risk_benchmarks {
            if percentage >= benchmark.min as f32 && percentage <= benchmark.max as f32 {
                return benchmark.label.clone();
            }
        }
    }

    // Then national benchmarks
    if let Some(national_benchmarks) = &assessment.national_benchmarks {
        for benchmark in national_benchmarks {
            if percentage >= benchmark.min as f32 && percentage <= benchmark.max as f32 {
                return benchmark.label.clone();
            }
        }
    }

    "Not Rated".to_string()
}

// Keep these utility functions for backwards compatibility if needed elsewhere
#[cfg(feature = "ssr")]
fn group_tests_by_name(
    test_history: &[TestHistoryEntry],
) -> HashMap<String, Vec<&TestHistoryEntry>> {
    let mut grouped_tests: HashMap<String, Vec<&TestHistoryEntry>> = HashMap::new();

    for entry in test_history {
        grouped_tests
            .entry(entry.test_name.clone())
            .or_insert_with(Vec::new)
            .push(entry);
    }

    for entries in grouped_tests.values_mut() {
        entries.sort_by(|a, b| a.date_administered.cmp(&b.date_administered));
    }

    grouped_tests
}

#[cfg(feature = "ssr")]
fn group_tests_by_name_and_attempt(
    test_history: &[TestHistoryEntry],
) -> HashMap<(String, i32), Vec<&TestHistoryEntry>> {
    let mut grouped_tests: HashMap<(String, i32), Vec<&TestHistoryEntry>> = HashMap::new();

    for entry in test_history {
        grouped_tests
            .entry((entry.test_name.clone(), entry.attempt))
            .or_insert_with(Vec::new)
            .push(entry);
    }

    for entries in grouped_tests.values_mut() {
        entries.sort_by(|a, b| a.date_administered.cmp(&b.date_administered));
    }

    grouped_tests
}

#[cfg(feature = "ssr")]
fn resolve_evaluator_name(evaluator_id: &str, teacher_lookup: &HashMap<String, String>) -> String {
    if evaluator_id.is_empty() {
        log::debug!("Evaluator ID is empty, returning 'Unknown'");
        return "Unknown".to_string();
    }

    if let Some(teacher_name) = teacher_lookup.get(evaluator_id) {
        log::debug!("Found teacher: {} -> {}", evaluator_id, teacher_name);
        return teacher_name.clone();
    }

    // If direct lookup fails, try parsing as integer and converting back to string
    if let Ok(id_num) = evaluator_id.trim().parse::<i32>() {
        let normalized_id = id_num.to_string();
        if let Some(teacher_name) = teacher_lookup.get(&normalized_id) {
            log::debug!(
                "Found teacher after normalization: {} -> {} -> {}",
                evaluator_id,
                normalized_id,
                teacher_name
            );
            return teacher_name.clone();
        }
    }

    // Return a descriptive fallback
    if evaluator_id.chars().all(|c| c.is_ascii_digit()) {
        format!("Teacher #{}", evaluator_id)
    } else {
        evaluator_id.to_string()
    }
}
