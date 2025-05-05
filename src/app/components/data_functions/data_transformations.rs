use crate::app::models::assessment::Assessment;
use crate::app::models::score::Score;
use crate::app::models::test::Test;
use polars::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

// Helper module for Polars-related data transformations
pub mod data_transformations {
    use super::*;

    /// Convert student scores to a Polars DataFrame
    pub fn scores_to_df(scores: &[Score]) -> Option<DataFrame> {
        if scores.is_empty() {
            return None;
        }

        // Extract data from scores
        let student_ids: Vec<i32> = scores.iter().map(|s| s.student_id).collect();
        let test_ids: Vec<String> = scores.iter().map(|s| s.test_id.clone()).collect();
        let totals: Vec<i32> = scores.iter().map(|s| s.get_total()).collect();
        let dates: Vec<Option<String>> = scores.iter().map(|s| s.date_administered.clone()).collect();

        // Create DataFrame
        let df = DataFrame::new(vec![
            Series::new("student_id", student_ids),
            Series::new("test_id", test_ids),
            Series::new("total", totals),
            Series::new("date", dates),
        ]).ok()?;

        Some(df)
    }

    /// Calculate basic statistics from scores
    pub fn calculate_score_statistics(scores: &[Score]) -> (i32, i32, i32) {
        if let Some(df) = scores_to_df(scores) {
            // Use Polars to calculate statistics
            let avg = df.column("total")
                .ok()
                .and_then(|s| s.mean())
                .map(|m| m as i32)
                .unwrap_or(0);
            
            let max = df.column("total")
                .ok()
                .and_then(|s| s.max())
                .map(|m| m as i32)
                .unwrap_or(0);
                
            let min = df.column("total")
                .ok()
                .and_then(|s| s.min())
                .map(|m| m as i32)
                .unwrap_or(0);

            (avg, max, min)
        } else {
            // Fallback to standard calculation if DataFrame creation failed
            if scores.is_empty() {
                return (0, 0, 0);
            }
            
            let total: i32 = scores.iter().map(|s| s.get_total()).sum();
            let count = scores.len() as i32;
            let avg = if count > 0 { total / count } else { 0 };
            let max = scores.iter().map(|s| s.get_total()).max().unwrap_or(0);
            let min = scores.iter().map(|s| s.get_total()).min().unwrap_or(0);
            
            (avg, max, min)
        }
    }

    /// Group scores by assessment
    pub fn group_scores_by_assessment(
        scores: &[Score], 
        tests: &[Test], 
        assessments: &[Assessment]
    ) -> HashMap<String, Vec<(Score, Test)>> {
        // Create a map of test_id to Assessment
        let mut test_to_assessment_map = HashMap::new();
        for assessment in assessments.iter() {
            for test_id in &assessment.tests {
                test_to_assessment_map.insert(test_id.clone(), assessment);
            }
        }

        // Group scores by assessment
        let mut result = HashMap::new();
        for score in scores.iter() {
            if let Some(test) = tests.iter().find(|t| t.test_id == score.test_id) {
                if let Some(assessment) = test_to_assessment_map.get(
                    &Uuid::parse_str(&score.test_id).expect("Failed conversion string -> Uuid"),
                ) {
                    let entry = result
                        .entry(assessment.id.to_string())
                        .or_insert_with(Vec::new);
                    entry.push((score.clone(), test.clone()));
                }
            }
        }

        result
    }

    /// Calculate average score for an assessment
    pub fn calculate_assessment_avg(assessment_scores: &[(Score, Test)]) -> i32 {
        if assessment_scores.is_empty() {
            return 0;
        }
        
        // Extract score totals
        let totals: Vec<i32> = assessment_scores.iter().map(|(score, _)| score.get_total()).collect();
        
        // Create a Series and calculate mean
        if let Ok(series) = Series::new("totals", totals) {
            series.mean().map(|m| m as i32).unwrap_or(0)
        } else {
            // Fallback to manual calculation
            let total: i32 = assessment_scores.iter().map(|(score, _)| score.get_total()).sum();
            let count = assessment_scores.len() as i32;
            if count > 0 {
                total / count
            } else {
                0
            }
        }
    }

    /// Generate score distribution data for charting
    pub fn generate_score_distribution(scores: &[Score]) -> Option<DataFrame> {
        if scores.is_empty() {
            return None;
        }

        // Create a DataFrame from scores
        let df = scores_to_df(scores)?;
        
        // Create score ranges
        let ranges = vec![
            (0, 20, "0-20"),
            (21, 40, "21-40"),
            (41, 60, "41-60"),
            (61, 80, "61-80"),
            (81, 100, "81-100"),
        ];
        
        // Count scores in each range
        let mut range_counts = vec![0; ranges.len()];
        if let Ok(totals_series) = df.column("total") {
            for total in totals_series.i32()?.into_iter().flatten() {
                for (i, (min, max, _)) in ranges.iter().enumerate() {
                    if total >= *min && total <= *max {
                        range_counts[i] += 1;
                        break;
                    }
                }
            }
        }
        
        // Create distribution DataFrame
        let range_labels: Vec<String> = ranges.iter().map(|(_, _, label)| label.to_string()).collect();
        
        DataFrame::new(vec![
            Series::new("range", range_labels),
            Series::new("count", range_counts),
        ]).ok()
    }
}
