use crate::app::models::assessment_sequences::{
    SequenceBehavior, TestSequenceItem, VariationLevel,
};
use crate::app::models::student::GradeEnum;
use crate::app::models::test::Test;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use std::str::FromStr;
use strum_macros::{EnumIter, EnumString};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone)]
pub struct AssessmentProgressionStatus {
    pub total_main_tests: usize,
    pub completed_main_tests: usize,
    pub completion_percentage: f32,
    pub variation_attempts: usize,
    pub intervention_needed: Vec<Uuid>,
    pub is_complete: bool,
}

impl AssessmentProgressionStatus {
    pub fn legacy_mode() -> Self {
        AssessmentProgressionStatus {
            total_main_tests: 0,
            completed_main_tests: 0,
            completion_percentage: 0.0,
            variation_attempts: 0,
            intervention_needed: vec![],
            is_complete: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RecommendedAction {
    TakeTest {
        test_id: Uuid,
        is_required: bool,
    },
    TakeVariation {
        main_test_id: Uuid,
        variation_test_id: Uuid,
        level: i32,
        description: String,
    },
    TakeOptionalTest {
        test_id: Uuid,
    },
    TakeDiagnostic {
        test_id: Uuid,
    },
    RetakeTest {
        test_id: Uuid,
        attempts_remaining: i32,
    },
    NeedsIntervention {
        test_id: Uuid,
        reason: String,
    },
    AssessmentComplete,
    LegacyMode,
}

#[derive(Debug, Clone)]
pub struct InterventionAlert {
    pub test_id: Uuid,
    pub alert_type: InterventionType,
    pub main_score: i32,
    pub variation_scores: Vec<(i32, i32)>, // (level, score) pairs
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum InterventionType {
    AllVariationsFailed,
    ExcessiveAttempts,
    StuckOnVariation,
    LowProgressRate,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, EnumIter)]
pub enum ScopeEnum {
    #[strum(to_string = "course")]
    Course,
    #[strum(to_string = "grade_level")]
    GradeLevel,
    #[strum(to_string = "all-required")]
    AllRequired,
}
impl fmt::Display for ScopeEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ScopeEnum::Course => "course".to_string(),
                ScopeEnum::GradeLevel => "grade_level".to_string(),
                ScopeEnum::AllRequired => "all-required".to_string(),
            }
        )
    }
}
impl FromStr for ScopeEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "course" => Ok(ScopeEnum::Course),
            "grade_level" => Ok(ScopeEnum::GradeLevel),
            "all-required" => Ok(ScopeEnum::AllRequired),
            _ => Err(format!("Invalid scope value: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RangeCategory {
    pub min: i32,
    pub max: i32,
    pub label: String,
}
impl RangeCategory {
    pub fn new(min: i32, max: i32, label: String) -> RangeCategory {
        RangeCategory { min, max, label }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, EnumIter, Copy)]
pub enum SubjectEnum {
    Reading,
    Math,
    Literacy,
    Phonics,
    History,
    Science,
    SocialStudies,
    Other,
}
impl fmt::Display for SubjectEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SubjectEnum::Reading => "Reading".to_string(),
                SubjectEnum::Math => "Math".to_string(),
                SubjectEnum::Literacy => "Literacy".to_string(),
                SubjectEnum::Phonics => "Phonics".to_string(),
                SubjectEnum::History => "History".to_string(),
                SubjectEnum::Science => "Science".to_string(),
                SubjectEnum::SocialStudies => "Social Studies".to_string(),
                SubjectEnum::Other => "Other".to_string(),
            }
        )
    }
}
impl FromStr for SubjectEnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Reading" => Ok(SubjectEnum::Reading),
            "Math" => Ok(SubjectEnum::Math),
            "Literacy" => Ok(SubjectEnum::Literacy),
            "Phonics" => Ok(SubjectEnum::Phonics),
            "History" => Ok(SubjectEnum::History),
            "Science" => Ok(SubjectEnum::Science),
            "Social Studies" => Ok(SubjectEnum::SocialStudies),
            "Other" => Ok(SubjectEnum::Other),
            _ => Err(format!("Invalid subject value: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Assessment {
    pub name: String,
    pub frequency: Option<i32>,
    pub grade: Option<GradeEnum>,
    pub version: i32,
    pub id: Uuid,
    pub tests: Vec<Uuid>,
    pub composite_score: Option<i32>,
    pub risk_benchmarks: Option<Vec<RangeCategory>>,
    pub national_benchmarks: Option<Vec<RangeCategory>>,
    pub subject: Option<SubjectEnum>,
    pub scope: Option<ScopeEnum>,
    pub course_id: Option<i32>,
    pub test_sequence: Option<Vec<TestSequenceItem>>,
}
impl Assessment {
    // Get the next test in sequence by order
    pub fn get_next_in_sequence(&self, current_order: i32) -> Option<Uuid> {
        let sequence = self.test_sequence.as_ref()?;

        sequence
            .iter()
            .find(|item| item.sequence_order == current_order + 1)
            .map(|item| item.test_id)
    }

    /// Get the next variation level for a failed test
    pub fn get_next_variation_level(
        &self,
        main_test_id: Uuid,
        student_history: &[(Uuid, i32)],
        main_item: &TestSequenceItem,
    ) -> Option<Uuid> {
        let variations = main_item.variation_levels.as_ref()?;

        let attempted_tests: std::collections::HashSet<Uuid> = student_history
            .iter()
            .map(|(test_id, _)| *test_id)
            .collect();

        for level in 1..=3 {
            if let Some(variation) = variations.iter().find(|v| v.level == level) {
                if !attempted_tests.contains(&variation.test_id) {
                    return Some(variation.test_id);
                }
            }
        }

        None
    }

    /// Legacy get_next_test method for backward compatibility
    pub fn get_next_test(&self, current_test_id: Uuid, score: Option<i32>) -> Option<Uuid> {
        let sequence = self.test_sequence.as_ref()?;
        let current_item = sequence
            .iter()
            .find(|item| item.test_id == current_test_id)?;

        match current_item.sequence_behavior {
            SequenceBehavior::Node => self.get_next_in_sequence(current_item.sequence_order),
            SequenceBehavior::Attainment => {
                let score = score?;
                let required_score = current_item.required_score.unwrap_or(70);

                if score >= required_score {
                    current_item
                        .next_on_pass
                        .or_else(|| self.get_next_in_sequence(current_item.sequence_order))
                } else {
                    current_item
                        .next_on_fail
                        .or_else(|| self.get_next_in_sequence(current_item.sequence_order))
                }
            }
            SequenceBehavior::Branching => {
                if let Some(score) = score {
                    if let Some(ranges) = &current_item.score_ranges {
                        for range in ranges {
                            if score >= range.min && score <= range.max {
                                return range.next_test;
                            }
                        }
                    }
                }
                self.get_next_in_sequence(current_item.sequence_order)
            }
            _ => self.get_next_in_sequence(current_item.sequence_order),
        }
    }

    /// Create a new Assessment with legacy test list (no sequences)
    pub fn new(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        id: Uuid,
        tests: Vec<Uuid>,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: Option<SubjectEnum>,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
    ) -> Assessment {
        Assessment {
            name,
            frequency,
            grade,
            version,
            id,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: None,
        }
    }
    pub fn new_with_sequence(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        id: Uuid,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: Option<SubjectEnum>,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
        test_sequence: Vec<TestSequenceItem>,
    ) -> Assessment {
        // Extract ALL test IDs including variations for backward compatibility
        let mut tests = Vec::new();

        for item in &test_sequence {
            // Add main test ID
            tests.push(item.test_id);

            // Add variation test IDs if they exist
            if let Some(variations) = &item.variation_levels {
                for variation in variations {
                    tests.push(variation.test_id);
                }
            }
        }

        Assessment {
            name,
            frequency,
            grade,
            version,
            id,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: Some(test_sequence),
        }
    }

    /// Get the next test with comprehensive multi-level variation logic
    pub fn get_next_test_adaptive(
        &self,
        current_test_id: Uuid,
        score: Option<i32>,
        student_history: &[(Uuid, i32)],
        all_tests: &[Test],
    ) -> Option<Uuid> {
        let sequence = self.test_sequence.as_ref()?;

        // Check if current test is a variation test
        if let Some(main_test_id) = self.find_main_test_for_variation(current_test_id) {
            return self.handle_variation_completion(
                main_test_id,
                current_test_id,
                score?,
                student_history,
            );
        }

        // Handle main test completion
        let current_item = sequence
            .iter()
            .find(|item| item.test_id == current_test_id)?;

        match current_item.sequence_behavior {
            SequenceBehavior::Attainment => {
                let score = score?;
                let required_score = current_item.required_score?;

                if score >= required_score {
                    // Passed main test: continue to next in sequence
                    current_item
                        .next_on_pass
                        .or_else(|| self.get_next_in_sequence(current_item.sequence_order))
                } else {
                    // Failed main test: try variations
                    self.get_next_variation_level(current_test_id, student_history, current_item)
                        .or_else(|| {
                            // No more variations: use fallback or continue
                            current_item
                                .next_on_fail
                                .or_else(|| self.get_next_in_sequence(current_item.sequence_order))
                        })
                }
            }
            _ => {
                // Other behaviors use existing logic
                self.get_next_test(current_test_id, score)
            }
        }
    }

    /// Find the main test ID for a variation test
    fn find_main_test_for_variation(&self, variation_test_id: Uuid) -> Option<Uuid> {
        let sequence = self.test_sequence.as_ref()?;

        for item in sequence {
            if let Some(variations) = &item.variation_levels {
                if variations.iter().any(|v| v.test_id == variation_test_id) {
                    return Some(item.test_id);
                }
            }
        }
        None
    }

    /// Handle completion of a variation test
    fn handle_variation_completion(
        &self,
        main_test_id: Uuid,
        variation_test_id: Uuid,
        score: i32,
        student_history: &[(Uuid, i32)],
    ) -> Option<Uuid> {
        let sequence = self.test_sequence.as_ref()?;
        let main_item = sequence.iter().find(|item| item.test_id == main_test_id)?;
        let variations = main_item.variation_levels.as_ref()?;

        let current_variation = variations.iter().find(|v| v.test_id == variation_test_id)?;
        let required_score = current_variation.required_score.unwrap_or(60);

        if score >= required_score {
            // Passed variation: continue to next test in main sequence
            main_item
                .next_on_pass
                .or_else(|| self.get_next_in_sequence(main_item.sequence_order))
        } else {
            // Failed variation: try next level or end
            let next_level = current_variation.level + 1;

            if let Some(next_variation) = variations.iter().find(|v| v.level == next_level) {
                // Check if this level has already been attempted
                let attempted = student_history
                    .iter()
                    .any(|(id, _)| *id == next_variation.test_id);
                if !attempted {
                    return Some(next_variation.test_id);
                }
            }

            // No more variations available: continue or flag for intervention
            main_item
                .next_on_pass
                .or_else(|| self.get_next_in_sequence(main_item.sequence_order))
        }
    }

    /// Get detailed progression status for a student
    pub fn get_progression_status(
        &self,
        student_history: &[(Uuid, i32)],
        all_tests: &[Test],
    ) -> AssessmentProgressionStatus {
        let sequence = match self.test_sequence.as_ref() {
            Some(seq) => seq,
            None => return AssessmentProgressionStatus::legacy_mode(),
        };

        let mut completed_main_tests = 0;
        let mut total_main_tests = 0;
        let mut variation_attempts = 0;
        let mut intervention_needed = Vec::new();
        let completed_tests: std::collections::HashMap<Uuid, i32> =
            student_history.iter().cloned().collect();

        for item in sequence {
            match item.sequence_behavior {
                SequenceBehavior::Attainment | SequenceBehavior::Node => {
                    total_main_tests += 1;

                    if let Some(&score) = completed_tests.get(&item.test_id) {
                        if item.sequence_behavior == SequenceBehavior::Attainment {
                            let required = item.required_score.unwrap_or(70);
                            if score >= required {
                                completed_main_tests += 1;
                            } else {
                                // Check variation attempts
                                if let Some(variations) = &item.variation_levels {
                                    let mut attempted_all_variations = true;
                                    let mut passed_any_variation = false;

                                    for variation in variations {
                                        if let Some(&var_score) =
                                            completed_tests.get(&variation.test_id)
                                        {
                                            variation_attempts += 1;
                                            let var_required =
                                                variation.required_score.unwrap_or(60);
                                            if var_score >= var_required {
                                                passed_any_variation = true;
                                                completed_main_tests += 1;
                                                break;
                                            }
                                        } else {
                                            attempted_all_variations = false;
                                            break;
                                        }
                                    }

                                    if attempted_all_variations && !passed_any_variation {
                                        intervention_needed.push(item.test_id);
                                    }
                                }
                            }
                        } else {
                            completed_main_tests += 1;
                        }
                    }
                }
                _ => {} // Skip optional, diagnostic, etc. for main progress
            }
        }

        let completion_percentage = if total_main_tests > 0 {
            (completed_main_tests as f32 / total_main_tests as f32) * 100.0
        } else {
            0.0
        };

        AssessmentProgressionStatus {
            total_main_tests,
            completed_main_tests,
            completion_percentage,
            variation_attempts,
            intervention_needed,
            is_complete: completed_main_tests >= total_main_tests,
        }
    }

    /// Get recommended next action for a student
    pub fn get_recommended_action(
        &self,
        student_history: &[(Uuid, i32)],
        all_tests: &[Test],
    ) -> RecommendedAction {
        let sequence = match self.test_sequence.as_ref() {
            Some(seq) => seq,
            None => return RecommendedAction::LegacyMode,
        };

        let completed_tests: std::collections::HashMap<Uuid, i32> =
            student_history.iter().cloned().collect();

        // Find the first incomplete main test
        for item in sequence {
            match item.sequence_behavior {
                SequenceBehavior::Attainment | SequenceBehavior::Node => {
                    if let Some(&score) = completed_tests.get(&item.test_id) {
                        // Test attempted - check if passed
                        if item.sequence_behavior == SequenceBehavior::Attainment {
                            let required = item.required_score.unwrap_or(70);
                            if score < required {
                                // Failed main test - check variations
                                if let Some(variations) = &item.variation_levels {
                                    // Sort variations by level to ensure proper order
                                    let sorted_variations: Vec<&VariationLevel> =
                                        variations.iter().sorted_by_key(|v| v.level).collect();

                                    for variation in sorted_variations {
                                        if !completed_tests.contains_key(&variation.test_id) {
                                            return RecommendedAction::TakeVariation {
                                                main_test_id: item.test_id,
                                                variation_test_id: variation.test_id,
                                                level: variation.level,
                                                description: variation.description.clone(),
                                            };
                                        }
                                    }
                                    // All variations attempted and failed
                                    return RecommendedAction::NeedsIntervention {
                                        test_id: item.test_id,
                                        reason: "Failed main test and all variations".to_string(),
                                    };
                                } else {
                                    // No variations available
                                    return RecommendedAction::RetakeTest {
                                        test_id: item.test_id,
                                        attempts_remaining: item.max_attempts.unwrap_or(1) - 1,
                                    };
                                }
                            }
                        }
                        // Test passed, continue to next
                    } else {
                        // Test not attempted yet
                        return RecommendedAction::TakeTest {
                            test_id: item.test_id,
                            is_required: item.sequence_behavior == SequenceBehavior::Attainment,
                        };
                    }
                }
                SequenceBehavior::Optional => {
                    if !completed_tests.contains_key(&item.test_id) {
                        return RecommendedAction::TakeOptionalTest {
                            test_id: item.test_id,
                        };
                    }
                }
                SequenceBehavior::Diagnostic => {
                    if !completed_tests.contains_key(&item.test_id) {
                        return RecommendedAction::TakeDiagnostic {
                            test_id: item.test_id,
                        };
                    }
                }
                _ => {} // Skip other types for now
            }
        }

        RecommendedAction::AssessmentComplete
    }

    /// Check if a student needs teacher intervention
    pub fn needs_intervention(&self, student_history: &[(Uuid, i32)]) -> Vec<InterventionAlert> {
        let sequence = match self.test_sequence.as_ref() {
            Some(seq) => seq,
            None => return vec![],
        };

        let mut alerts = Vec::new();
        let completed_tests: std::collections::HashMap<Uuid, i32> =
            student_history.iter().cloned().collect();

        for item in sequence {
            if item.sequence_behavior == SequenceBehavior::Attainment {
                if let Some(&main_score) = completed_tests.get(&item.test_id) {
                    let required = item.required_score.unwrap_or(70);

                    if main_score < required {
                        // Check if all variations have been attempted and failed
                        if let Some(variations) = &item.variation_levels {
                            let mut all_variations_failed = true;
                            let mut attempted_variations = 0;

                            for variation in variations {
                                if let Some(&var_score) = completed_tests.get(&variation.test_id) {
                                    attempted_variations += 1;
                                    let var_required = variation.required_score.unwrap_or(60);
                                    if var_score >= var_required {
                                        all_variations_failed = false;
                                        break;
                                    }
                                } else {
                                    all_variations_failed = false;
                                    break;
                                }
                            }

                            if all_variations_failed && attempted_variations == variations.len() {
                                alerts.push(InterventionAlert {
                                    test_id: item.test_id,
                                    alert_type: InterventionType::AllVariationsFailed,
                                    main_score,
                                    variation_scores: variations.iter()
                                        .filter_map(|v| completed_tests.get(&v.test_id).map(|&score| (v.level, score)))
                                        .collect(),
                                    message: format!(
                                        "Student failed main test ({}%) and all {} variation levels. Needs teacher support.",
                                        main_score, variations.len()
                                    ),
                                });
                            }
                        }

                        // Check for excessive attempts on main test
                        let main_attempts = student_history
                            .iter()
                            .filter(|(id, _)| *id == item.test_id)
                            .count();

                        if main_attempts >= item.max_attempts.unwrap_or(3) as usize {
                            alerts.push(InterventionAlert {
                                test_id: item.test_id,
                                alert_type: InterventionType::ExcessiveAttempts,
                                main_score,
                                variation_scores: vec![],
                                message: format!(
                                    "Student has used all {} attempts on main test with best score of {}%",
                                    main_attempts, main_score
                                ),
                            });
                        }
                    }
                }
            }
        }

        alerts
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, Clone)]
pub struct CreateNewAssessmentRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub frequency: Option<i32>,
    pub grade: Option<GradeEnum>,
    pub version: i32,
    pub tests: Vec<Uuid>,
    pub composite_score: Option<i32>,
    pub risk_benchmarks: Option<Vec<RangeCategory>>,
    pub national_benchmarks: Option<Vec<RangeCategory>>,
    pub subject: Option<SubjectEnum>,
    pub scope: Option<ScopeEnum>,
    pub course_id: Option<i32>,
    pub test_sequence: Option<Vec<TestSequenceItem>>,
}
impl CreateNewAssessmentRequest {
    pub fn new(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        tests: Vec<Uuid>,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: Option<SubjectEnum>,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
    ) -> CreateNewAssessmentRequest {
        CreateNewAssessmentRequest {
            name,
            frequency,
            grade,
            version,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: None,
        }
    }
    pub fn new_with_sequence(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: Option<SubjectEnum>,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
        test_sequence: Vec<TestSequenceItem>,
    ) -> CreateNewAssessmentRequest {
        // Extract ALL test IDs including variations for backward compatibility
        let mut tests = Vec::new();

        for item in &test_sequence {
            tests.push(item.test_id);
            if let Some(variations) = &item.variation_levels {
                for variation in variations {
                    tests.push(variation.test_id);
                }
            }
        }

        CreateNewAssessmentRequest {
            name,
            frequency,
            grade,
            version,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: Some(test_sequence),
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize, Clone)]
pub struct UpdateAssessmentRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub frequency: Option<i32>,
    pub grade: Option<GradeEnum>,
    pub version: i32,
    pub id: Uuid,
    pub tests: Vec<Uuid>,
    pub composite_score: Option<i32>,
    pub risk_benchmarks: Option<Vec<RangeCategory>>,
    pub national_benchmarks: Option<Vec<RangeCategory>>,
    pub subject: Option<SubjectEnum>,
    pub scope: Option<ScopeEnum>,
    pub course_id: Option<i32>,
    pub test_sequence: Option<Vec<TestSequenceItem>>,
}
impl UpdateAssessmentRequest {
    pub fn new(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        id: Uuid,
        tests: Vec<Uuid>,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: Option<SubjectEnum>,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
    ) -> UpdateAssessmentRequest {
        UpdateAssessmentRequest {
            name,
            frequency,
            grade,
            version,
            id,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: None,
        }
    }
    pub fn new_with_sequence(
        name: String,
        frequency: Option<i32>,
        grade: Option<GradeEnum>,
        version: i32,
        id: Uuid,
        composite_score: Option<i32>,
        risk_benchmarks: Option<Vec<RangeCategory>>,
        national_benchmarks: Option<Vec<RangeCategory>>,
        subject: Option<SubjectEnum>,
        scope: Option<ScopeEnum>,
        course_id: Option<i32>,
        test_sequence: Vec<TestSequenceItem>,
    ) -> UpdateAssessmentRequest {
        // Extract ALL test IDs including variations for backward compatibility
        let mut tests = Vec::new();

        for item in &test_sequence {
            tests.push(item.test_id);
            if let Some(variations) = &item.variation_levels {
                for variation in variations {
                    tests.push(variation.test_id);
                }
            }
        }

        UpdateAssessmentRequest {
            name,
            frequency,
            grade,
            version,
            id,
            tests,
            composite_score,
            risk_benchmarks,
            national_benchmarks,
            subject,
            scope,
            course_id,
            test_sequence: Some(test_sequence),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeleteAssessmentRequest {
    pub version: i32,
    pub id: Uuid,
}
impl DeleteAssessmentRequest {
    pub fn new(version: i32, id: Uuid) -> DeleteAssessmentRequest {
        DeleteAssessmentRequest { version, id }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Postgres, Encode, Decode, Type, postgres::{PgTypeInfo, PgValueRef, PgArgumentBuffer, PgHasArrayType}, encode::IsNull};
        use sqlx::prelude::*;
        use sqlx::types::Json;

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for SubjectEnum {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for SubjectEnum {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                SubjectEnum::from_str(s).map_err(|_| format!("Invalid SubjectEnum: {}", s).into())
            }
        }
        impl Type<Postgres> for SubjectEnum {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("subject_enum")
            }
        }

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for RangeCategory {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                Json(self).encode_by_ref(buf)
            }
        }
        impl sqlx::Type<sqlx::Postgres> for RangeCategory {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("jsonb")
            }
        }
        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for RangeCategory {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let json: Json<RangeCategory> = sqlx::decode::Decode::decode(value)?;
                Ok(json.0)
            }
        }
        impl sqlx::postgres::PgHasArrayType for RangeCategory {
            fn array_type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("_jsonb")
            }
        }
        impl <'q> sqlx::encode::Encode<'q, sqlx::Postgres> for ScopeEnum {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                let s = self.to_string();
                <&str as Encode<Postgres>>::encode(&s.as_str(), buf)
            }
        }
        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for ScopeEnum {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let s: &str = Decode::<sqlx::Postgres>::decode(value)?;
                ScopeEnum::from_str(s).map_err(|_| format!("Invalid ScopeEnum: {}", s).into())
            }
        }
        impl sqlx::Type<sqlx::Postgres> for ScopeEnum {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("assessment_scope_enum")
            }
        }

        ///Create wrapper for new type Vec<RangeCategory> to solve orphan rule issue in rust
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
        pub struct RangeCategoriesWrapper(pub Vec<RangeCategory>);

        impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for RangeCategoriesWrapper {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
                Json(&self.0).encode_by_ref(buf)
            }
        }

        impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for RangeCategoriesWrapper {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                let json: Json<Vec<RangeCategory>> = sqlx::decode::Decode::decode(value)?;
                Ok(RangeCategoriesWrapper(json.0))
            }
        }
    }
}
