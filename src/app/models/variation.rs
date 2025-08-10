use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use crate::app::models::test::Test;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariationType {
    Randomized,
    Distinct, 
    Practice,
}

impl VariationType {
    pub fn display_name(&self) -> &'static str {
        match self {
            VariationType::Randomized => "Randomized",
            VariationType::Distinct => "Distinct",
            VariationType::Practice => "Practice",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            VariationType::Randomized => "Same questions with shuffled order and randomized answer choices",
            VariationType::Distinct => "Entirely different questions covering the same topics",
            VariationType::Practice => "Practice version for student preparation with new questions",
        }
    }

    pub fn detailed_description(&self) -> &'static str {
        match self {
            VariationType::Randomized => "Creates a test variation with the same questions but in randomized order with shuffled answer choices. Questions are automatically generated from the base test.",
            VariationType::Distinct => "Creates a blank test variation where you can add entirely new questions for a different version covering the same material.",
            VariationType::Practice => "Creates a blank practice version where you can add new questions for student practice and preparation.",
        }
    }

    pub fn variant_number_offset(&self) -> i32 {
        match self {
            VariationType::Randomized => 100,
            VariationType::Distinct => 200,
            VariationType::Practice => 300,
        }
    }

    pub fn badge_class(&self) -> &'static str {
        match self {
            VariationType::Randomized => "bg-blue-100 text-blue-800 border-blue-300",
            VariationType::Distinct => "bg-green-100 text-green-800 border-green-300",
            VariationType::Practice => "bg-purple-100 text-purple-800 border-purple-300",
        }
    }

    pub fn card_class(&self) -> &'static str {
        match self {
            VariationType::Randomized => "border-blue-200 bg-blue-50",
            VariationType::Distinct => "border-green-200 bg-green-50",
            VariationType::Practice => "border-purple-200 bg-purple-50",
        }
    }

    pub fn requires_manual_questions(&self) -> bool {
        match self {
            VariationType::Randomized => false,
            VariationType::Distinct => true,
            VariationType::Practice => true,
        }
    }

    pub fn from_test_name(test_name: &str) -> Option<Self> {
        let name_lower = test_name.to_lowercase();
        if name_lower.contains("randomized") {
            Some(VariationType::Randomized)
        } else if name_lower.contains("distinct") {
            Some(VariationType::Distinct)
        } else if name_lower.contains("practice") {
            Some(VariationType::Practice)
        } else {
            None
        }
    }

    pub fn from_comments(comments: &str) -> Option<Self> {
        let comments_lower = comments.to_lowercase();
        if comments_lower.contains("variation: randomized") {
            Some(VariationType::Randomized)
        } else if comments_lower.contains("variation: distinct") {
            Some(VariationType::Distinct)
        } else if comments_lower.contains("variation: practice") {
            Some(VariationType::Practice)
        } else {
            None
        }
    }
}

impl Display for VariationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestVariationInfo {
    pub base_test: Test,
    pub randomized_variations: Vec<Test>,
    pub distinct_variations: Vec<Test>,
    pub practice_variations: Vec<Test>,
}

impl TestVariationInfo {
    pub fn new(base_test: Test) -> Self {
        Self {
            base_test,
            randomized_variations: Vec::new(),
            distinct_variations: Vec::new(),
            practice_variations: Vec::new(),
        }
    }

    pub fn add_variation(&mut self, variation: Test) {
        if let Some(var_type) = VariationType::from_test_name(&variation.name) {
            match var_type {
                VariationType::Randomized => self.randomized_variations.push(variation),
                VariationType::Distinct => self.distinct_variations.push(variation),
                VariationType::Practice => self.practice_variations.push(variation),
            }
        } else if let Some(var_type) = VariationType::from_comments(&variation.comments) {
            match var_type {
                VariationType::Randomized => self.randomized_variations.push(variation),
                VariationType::Distinct => self.distinct_variations.push(variation),
                VariationType::Practice => self.practice_variations.push(variation),
            }
        }
    }

    pub fn total_variations(&self) -> usize {
        self.randomized_variations.len() + self.distinct_variations.len() + self.practice_variations.len()
    }

    pub fn get_variations_by_type(&self, var_type: &VariationType) -> &Vec<Test> {
        match var_type {
            VariationType::Randomized => &self.randomized_variations,
            VariationType::Distinct => &self.distinct_variations,
            VariationType::Practice => &self.practice_variations,
        }
    }

    pub fn has_variation_type(&self, var_type: &VariationType) -> bool {
        !self.get_variations_by_type(var_type).is_empty()
    }

    pub fn get_all_variations(&self) -> Vec<&Test> {
        let mut all = Vec::new();
        all.extend(self.randomized_variations.iter());
        all.extend(self.distinct_variations.iter());
        all.extend(self.practice_variations.iter());
        all
    }
}

// Utility functions for test variation detection
pub fn is_variation_test(test: &Test) -> bool {
    test.name.contains(" - ") && (
        VariationType::from_test_name(&test.name).is_some() ||
        VariationType::from_comments(&test.comments).is_some()
    )
}

pub fn get_base_test_name(test_name: &str) -> String {
    if test_name.contains(" - ") {
        test_name.split(" - ").next().unwrap_or(test_name).to_string()
    } else {
        test_name.to_string()
    }
}

pub fn get_variation_name_suffix(test_name: &str) -> Option<String> {
    if test_name.contains(" - ") {
        test_name.split(" - ").nth(1).map(|s| s.to_string())
    } else {
        None
    }
}

// Group tests into variation families
pub fn group_tests_by_base(tests: Vec<Test>) -> Vec<TestVariationInfo> {
    let mut groups: std::collections::HashMap<String, TestVariationInfo> = std::collections::HashMap::new();

    for test in tests {
        let base_name = get_base_test_name(&test.name);

        if is_variation_test(&test) {
            groups.entry(base_name.clone())
                .and_modify(|group| group.add_variation(test.clone()))
                .or_insert_with(|| {
                    let mut group = TestVariationInfo::new(test.clone());
                    // Clear the base test since we're adding a variation first
                    group.base_test = Test::new(
                        base_name.clone(),
                        0,
                        String::new(),
                        test.testarea.clone(),
                        test.school_year.clone(),
                        test.benchmark_categories.clone(),
                        0,
                        test.grade_level.clone(),
                        String::new(),
                        test.scope.clone(),
                        test.course_id.clone(),
                    );
                    group.add_variation(test.clone());
                    group
                });
        } else {
            groups.entry(base_name.clone())
                .and_modify(|group| {
                    // Replace placeholder base test if it exists
                    if group.base_test.test_id.is_empty() {
                        group.base_test = test.clone();
                    }
                })
                .or_insert_with(|| TestVariationInfo::new(test.clone()));
        }
    }

    // Filter out groups that only have placeholder base tests
    groups.into_values()
        .filter(|group| !group.base_test.test_id.is_empty())
        .collect()
}
