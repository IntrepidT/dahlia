use crate::app::models::assessment::Assessment;
use crate::app::models::score::Score;
use crate::app::models::student::Student;
use crate::app::models::test::Test;
use crate::app::server_functions::{
    assessments::get_assessments, scores::get_student_scores, students::get_student,
    tests::get_tests,
};
use chrono::prelude::*;
#[cfg(feature = "ssr")]
use polars::prelude::*;
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
    pub test_details: Vec<TestDetail>,
    pub distribution_data: Vec<(String, i32)>, //these should be overall performance metrics across
    //all test benchmarks for the specific assessment
    //(rating, score)
    pub assessment_rating: String, //this is where the student's overall rating is given according
    //to the assessment benchmarks
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
    pub performance_class: String, //this would be the categorization of the student according to
                                   //the benchmark categories for a specific test
}

#[cfg(feature = "ssr")]
pub fn tests_to_dataframe(tests: Vec<Test>) -> Result<DataFrame, PolarsError> {
    let names = tests.iter().map(|t| t.name.clone()).collect::<Vec<_>>();
    let score = tests.iter().map(|t| t.score).collect::<Vec<_>>();
    let testarea = tests
        .iter()
        .map(|t| t.testarea.to_string())
        .collect::<Vec<_>>();
    let school_year = tests
        .iter()
        .map(|t| t.school_year.clone())
        .collect::<Vec<_>>();

    // Convert complex optional types to strings
    let benchmark_categories = tests
        .iter()
        .map(|t| match &t.benchmark_categories {
            Some(cats) => format!("Categories: {}", cats.len()),
            None => "None".to_string(),
        })
        .collect::<Vec<_>>();

    let test_variant = tests.iter().map(|t| t.test_variant).collect::<Vec<_>>();
    let test_id = tests.iter().map(|t| t.test_id.clone()).collect::<Vec<_>>();

    df![
        "names" => names,
        "possible score" => score,
        "test area" => testarea,
        "school_year" => school_year,
        "benchmark_categories" => benchmark_categories,
        "test variant" => test_variant,
        "id" => test_id,
    ]
}

#[cfg(feature = "ssr")]
pub fn assessments_to_dataframe(assessments: Vec<Assessment>) -> Result<DataFrame, PolarsError> {
    let name = assessments
        .iter()
        .map(|a| a.name.clone())
        .collect::<Vec<_>>();
    let freq = assessments.iter().map(|a| a.frequency).collect::<Vec<_>>();

    // Convert GradeEnum to string representation
    let grade = assessments
        .iter()
        .map(|a| match &a.grade {
            Some(g) => g.to_string(),
            None => "None".to_string(),
        })
        .collect::<Vec<_>>();

    let version = assessments.iter().map(|a| a.version).collect::<Vec<_>>();
    let id = assessments
        .iter()
        .map(|a| a.id.to_string())
        .collect::<Vec<_>>(); // Convert UUID to string

    // Convert Vec<UUID> to string counts
    let tests = assessments
        .iter()
        .map(|a| format!("{} tests", a.tests.len()))
        .collect::<Vec<_>>();

    let composite_score = assessments
        .iter()
        .map(|a| a.composite_score)
        .collect::<Vec<_>>();

    // Convert benchmark types to string representations
    let risk_benchmarks = assessments
        .iter()
        .map(|a| match &a.risk_benchmarks {
            Some(benchmarks) => format!("{} benchmarks", benchmarks.len()),
            None => "None".to_string(),
        })
        .collect::<Vec<_>>();

    let national_benchmarks = assessments
        .iter()
        .map(|a| match &a.national_benchmarks {
            Some(benchmarks) => format!("{} benchmarks", benchmarks.len()),
            None => "None".to_string(),
        })
        .collect::<Vec<_>>();

    let subject = assessments
        .iter()
        .map(|a| a.subject.to_string())
        .collect::<Vec<_>>();

    df![
        "name" => name,
        "test frequency" => freq,
        "grade" => grade,
        "version" => version,
        "id" => id,
        "tests" => tests,
        "composite score" => composite_score,
        "risk benchmarks" => risk_benchmarks,
        "national benchmarks" => national_benchmarks,
        "subject" => subject,
    ]
}

#[cfg(feature = "ssr")]
pub fn scores_to_dataframe(scores: Vec<Score>) -> Result<DataFrame, PolarsError> {
    // Convert DateTime<Utc> to formatted strings for Polars compatibility
    let date = scores
        .iter()
        .map(|s| s.date_administered.to_rfc3339())
        .collect::<Vec<_>>();

    let test_id = scores.iter().map(|s| s.test_id.clone()).collect::<Vec<_>>();
    let test_scores = scores.iter().map(|s| s.get_total()).collect::<Vec<_>>();
    let comments = scores
        .iter()
        .map(|s| s.comments.join("; "))
        .collect::<Vec<_>>(); // Join comments for display
    let test_variant = scores.iter().map(|s| s.test_variant).collect::<Vec<_>>();
    let evaluator = scores
        .iter()
        .map(|s| s.evaluator.clone())
        .collect::<Vec<_>>();

    df![
        "test id" => test_id,
        "date" => date,
        "score" => test_scores,
        "comments" => comments,
        "test variant" => test_variant,
        "evaluator" => evaluator,
    ]
}

#[cfg(feature = "ssr")]
pub async fn get_student_results(student_id: i32) -> Result<StudentResultsSummary, String> {
    let tests = get_tests().await.unwrap();
    let assessments = get_assessments().await.unwrap();
    let scores = get_student_scores(student_id).await.unwrap();
    let student = get_student(student_id).await.map_err(|e| e.to_string())?;

    // Convert to dataframes for analysis
    let tests_df = tests_to_dataframe(tests.clone()).map_err(|e| e.to_string())?;
    let assessments_df =
        assessments_to_dataframe(assessments.clone()).map_err(|e| e.to_string())?;
    let scores_df = scores_to_dataframe(scores.clone()).map_err(|e| e.to_string())?;

    // Create a map of test_id to Test for easy lookup
    let test_map: HashMap<String, &Test> = tests
        .iter()
        .map(|test| (test.test_id.clone(), test))
        .collect();

    // Create a map of assessment_id to Assessment for easy lookup
    let assessment_map: HashMap<String, &Assessment> = assessments
        .iter()
        .map(|assessment| (assessment.id.to_string(), assessment))
        .collect();

    let mut test_history = Vec::new();
    for score in &scores {
        if let Some(test) = test_map.get(&score.test_id) {
            let score_total = score.get_total();
            let performance_class = determine_performance_class(test, score_total);

            test_history.push(TestHistoryEntry {
                test_id: score.test_id.clone(),
                test_name: test.name.clone(),
                score: score_total,
                total_possible: test.score,
                date_administered: score.date_administered,
                performance_class,
                evaluator: score.evaluator.clone(),
            });
        }
    }
    test_history.sort_by(|a, b| b.date_administered.cmp(&a.date_administered));

    // Find the highest score for each test
    let mut highest_scores: HashMap<String, &Score> = HashMap::new();
    for score in &scores {
        match highest_scores.get(&score.test_id) {
            Some(existing_score) if existing_score.get_total() >= score.get_total() => {
                // Keep existing highest score
            }
            _ => {
                // Update with new highest score
                highest_scores.insert(score.test_id.clone(), score);
            }
        }
    }

    // Process test details using only the highest scores
    let mut test_details = Vec::new();
    for (test_id, score) in &highest_scores {
        if let Some(test) = test_map.get(test_id) {
            let score_total = score.get_total();
            let performance_class = determine_performance_class(test, score_total);

            test_details.push(TestDetail {
                test_id: score.test_id.clone(),
                test_name: test.name.clone(),
                score: score_total,
                total_possible: test.score,
                test_area: test.testarea.to_string(),
                date_administered: score.date_administered,
                performance_class,
            });
        }
    }

    // Group test details by assessment
    let mut assessment_test_map: HashMap<String, Vec<TestDetail>> = HashMap::new();
    for test_detail in &test_details {
        // Find which assessment this test belongs to
        for (assessment_id, assessment) in &assessment_map {
            if assessment.tests.iter().any(|test_uuid| {
                // Convert the UUID to string for comparison
                test_uuid.to_string() == test_detail.test_id
            }) {
                assessment_test_map
                    .entry(assessment_id.clone())
                    .or_insert_with(Vec::new)
                    .push(test_detail.clone());
            }
        }
    }

    // Create assessment summaries
    let mut assessment_summaries = Vec::new();
    for (assessment_id, test_details) in &assessment_test_map {
        if let Some(assessment) = assessment_map.get(assessment_id) {
            let current_score = test_details.iter().map(|td| td.score).sum();
            let total_possible = assessment.composite_score;

            // Calculate distribution data and assessment rating
            let distribution_data = calculate_distribution_data(assessment, test_details);
            let assessment_rating =
                determine_assessment_rating(assessment, current_score, total_possible.unwrap_or(0));

            // Determine progress based on test completion
            let assessment_test_ids: Vec<String> = assessment
                .tests
                .iter()
                .map(|uuid| uuid.to_string())
                .collect();

            let completed_test_ids: Vec<String> =
                test_details.iter().map(|td| td.test_id.clone()).collect();

            let progress = if completed_test_ids.len() == assessment_test_ids.len()
                && assessment_test_ids
                    .iter()
                    .all(|id| completed_test_ids.contains(id))
            {
                Progress::Completed
            } else if completed_test_ids.is_empty() {
                Progress::NotStarted
            } else {
                Progress::Ongoing
            };

            assessment_summaries.push(AssessmentSummary {
                assessment_id: assessment_id.clone(),
                assessment_name: assessment.name.clone(),
                subject: assessment.subject.to_string(),
                total_possible,
                current_score,
                grade_level: assessment.grade.as_ref().map(|g| g.to_string()),
                test_details: test_details.clone(),
                distribution_data,
                assessment_rating,
                progress,
            });
        }
    }

    Ok(StudentResultsSummary {
        student,
        assessment_summaries,
        test_summaries: test_details,
        test_history,
    })
}

#[cfg(feature = "ssr")]
fn determine_performance_class(test: &Test, score: i32) -> String {
    // Logic to determine performance class based on test benchmark categories
    if let Some(benchmark_categories) = &test.benchmark_categories {
        let percentage = (score as f32 / test.score as f32) * 100.00;

        for benchmark in benchmark_categories {
            if percentage >= (benchmark.min as f32) && percentage <= (benchmark.max as f32) {
                return benchmark.label.clone();
            }
        }
    }
    "Not Rated".to_string()
}

#[cfg(feature = "ssr")]
fn calculate_distribution_data(
    assessment: &Assessment,
    test_details: &[TestDetail],
) -> Vec<(String, i32)> {
    // Calculate distribution of performance classifications
    let mut distribution: HashMap<String, i32> = HashMap::new();

    for test_detail in test_details {
        *distribution
            .entry(test_detail.performance_class.clone())
            .or_insert(0) += 1;
    }

    distribution.into_iter().collect()
}

#[cfg(feature = "ssr")]
fn determine_assessment_rating(assessment: &Assessment, score: i32, total_possible: i32) -> String {
    let score_percentage = (score as f32 / total_possible as f32) * 100.00;

    // First check risk benchmarks
    if let Some(risk_benchmarks) = &assessment.risk_benchmarks {
        for benchmark in risk_benchmarks {
            if score_percentage >= (benchmark.min as f32)
                && score_percentage <= (benchmark.max as f32)
            {
                return benchmark.label.clone();
            }
        }
    }

    // Then check national benchmarks
    if let Some(national_benchmarks) = &assessment.national_benchmarks {
        for benchmark in national_benchmarks {
            if score_percentage >= (benchmark.min as f32)
                && score_percentage <= (benchmark.max as f32)
            {
                return benchmark.label.clone();
            }
        }
    }

    "Not Rated".to_string()
}

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
