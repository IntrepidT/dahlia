use crate::app::models::test::BenchmarkCategory;

//This file contains utilities for working with benchmark categories (primarily used in
//test_builder.rs)

/// Utilities for working with benchmark categories in both single value and range formats
pub struct BenchmarkUtils;

impl BenchmarkUtils {
    /// Creates benchmark categories from the UI tuple representation (id, min, max, label)
    /// Automatically detects single values when min == max
    pub fn from_tuples(tuples: Vec<(i32, i32, i32, String, String)>) -> Vec<BenchmarkCategory> {
        tuples
            .into_iter()
            .map(|(_, min, max, label, color)| {
                let mut category = if min == max {
                    BenchmarkCategory::new_single(min, label)
                } else {
                    BenchmarkCategory::new_range(min, max, label)
                };
                category.color = Some(color);
                category
            })
            .collect()
    }

    /// Converts benchmark categories to tuples for UI consumption
    /// Returns (index, min, max, label) tuples
    pub fn to_tuples(categories: Vec<BenchmarkCategory>) -> Vec<(i32, i32, i32, String, String)> {
        categories
            .into_iter()
            .enumerate()
            .map(|(idx, cat)| {
                let color = cat.get_color();
                (idx as i32, cat.min, cat.max, cat.label, color)
            })
            .collect()
    }

    /// Finds which benchmark category a score belongs to
    /// Returns the first matching category (categories should not overlap)
    pub fn find_category_for_score(
        score: i32,
        categories: &[BenchmarkCategory],
    ) -> Option<&BenchmarkCategory> {
        categories.iter().find(|cat| cat.contains(score))
    }

    /// Gets the grade/label for a given score
    /// Returns None if no category matches the score
    pub fn get_grade_for_score(score: i32, categories: &[BenchmarkCategory]) -> Option<String> {
        Self::find_category_for_score(score, categories).map(|cat| cat.label.clone())
    }

    /// Validates benchmark categories for overlapping ranges
    /// Returns an error if any categories have overlapping score ranges
    pub fn validate_no_overlaps(categories: &[BenchmarkCategory]) -> Result<(), String> {
        if categories.is_empty() {
            return Ok(());
        }

        for (i, cat1) in categories.iter().enumerate() {
            for (j, cat2) in categories.iter().enumerate() {
                if i != j {
                    let overlap = Self::categories_overlap(cat1, cat2);

                    if overlap {
                        return Err(format!(
                            "Categories '{}' ({}) and '{}' ({}) have overlapping score ranges",
                            cat1.label,
                            cat1.range_display(),
                            cat2.label,
                            cat2.range_display()
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Checks if two benchmark categories have overlapping score ranges
    fn categories_overlap(cat1: &BenchmarkCategory, cat2: &BenchmarkCategory) -> bool {
        if cat1.is_single_value() && cat2.is_single_value() {
            // Both single values - overlap if they're the same value
            cat1.min == cat2.min
        } else if cat1.is_single_value() {
            // cat1 is single value, cat2 is range - overlap if cat2 contains cat1's value
            cat2.contains(cat1.min)
        } else if cat2.is_single_value() {
            // cat2 is single value, cat1 is range - overlap if cat1 contains cat2's value
            cat1.contains(cat2.min)
        } else {
            // Both are ranges - overlap if ranges intersect
            !(cat1.max < cat2.min || cat2.max < cat1.min)
        }
    }

    /// Validates that all categories have valid ranges (min <= max) and non-empty labels
    pub fn validate_categories(categories: &[BenchmarkCategory]) -> Result<(), String> {
        for cat in categories {
            if cat.label.trim().is_empty() {
                return Err("All benchmark categories must have a label".to_string());
            }

            if cat.min > cat.max {
                return Err(format!(
                    "Category '{}' has invalid range: min ({}) cannot be greater than max ({})",
                    cat.label, cat.min, cat.max
                ));
            }

            if cat.min < 0 {
                return Err(format!(
                    "Category '{}' has invalid minimum score: {} (scores cannot be negative)",
                    cat.label, cat.min
                ));
            }
        }

        Ok(())
    }

    /// Comprehensive validation that checks both individual categories and overlaps
    pub fn validate_all(categories: &[BenchmarkCategory]) -> Result<(), String> {
        Self::validate_categories(categories)?;
        Self::validate_no_overlaps(categories)?;
        Ok(())
    }

    /// Formats benchmark categories for display in summaries
    /// Returns a human-readable string representation
    pub fn format_summary(categories: &[BenchmarkCategory]) -> String {
        if categories.is_empty() {
            return "No benchmark categories defined".to_string();
        }

        let formatted: Vec<String> = categories
            .iter()
            .map(|cat| format!("{}: {}", cat.label, cat.range_display()))
            .collect();

        formatted.join(", ")
    }

    /// Sorts categories by their minimum score (useful for display)
    pub fn sort_by_min_score(mut categories: Vec<BenchmarkCategory>) -> Vec<BenchmarkCategory> {
        categories.sort_by_key(|cat| cat.min);
        categories
    }

    /// Groups categories by type (single values vs ranges)
    pub fn group_by_type(
        categories: &[BenchmarkCategory],
    ) -> (Vec<&BenchmarkCategory>, Vec<&BenchmarkCategory>) {
        let mut single_values = Vec::new();
        let mut ranges = Vec::new();

        for cat in categories {
            if cat.is_single_value() {
                single_values.push(cat);
            } else {
                ranges.push(cat);
            }
        }

        (single_values, ranges)
    }

    /// Gets statistics about the benchmark categories
    pub fn get_stats(categories: &[BenchmarkCategory]) -> BenchmarkStats {
        let total_count = categories.len();
        let (single_values, ranges) = Self::group_by_type(categories);
        let single_count = single_values.len();
        let range_count = ranges.len();

        let min_score = categories.iter().map(|cat| cat.min).min();
        let max_score = categories.iter().map(|cat| cat.max).max();

        BenchmarkStats {
            total_count,
            single_count,
            range_count,
            min_score,
            max_score,
        }
    }
}

/// Statistics about a set of benchmark categories
#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub total_count: usize,
    pub single_count: usize,
    pub range_count: usize,
    pub min_score: Option<i32>,
    pub max_score: Option<i32>,
}

impl BenchmarkStats {
    pub fn summary(&self) -> String {
        if self.total_count == 0 {
            return "No benchmark categories".to_string();
        }

        let mut parts = vec![format!("{} total", self.total_count)];

        if self.single_count > 0 {
            parts.push(format!(
                "{} single value{}",
                self.single_count,
                if self.single_count == 1 { "" } else { "s" }
            ));
        }

        if self.range_count > 0 {
            parts.push(format!(
                "{} range{}",
                self.range_count,
                if self.range_count == 1 { "" } else { "s" }
            ));
        }

        let type_info = parts.join(", ");

        match (self.min_score, self.max_score) {
            (Some(min), Some(max)) => {
                format!("{} (covering scores {}-{})", type_info, min, max)
            }
            _ => type_info,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_tuples() {
        let tuples = vec![
            (0, 90, 100, "A".to_string(), "#10b981".to_string()),
            (1, 85, 85, "Perfect".to_string(), "#ef4444".to_string()),
        ];

        let categories = BenchmarkUtils::from_tuples(tuples);

        assert_eq!(categories.len(), 2);
        assert!(!categories[0].is_single_value());
        assert!(categories[1].is_single_value());
        assert_eq!(categories[0].get_color(), "#10b981");
        assert_eq!(categories[1].get_color(), "#ef4444");
    }

    #[test]
    fn test_validate_overlapping_ranges() {
        let categories = vec![
            BenchmarkCategory::new_range(70, 80, "B".to_string()),
            BenchmarkCategory::new_range(75, 85, "A".to_string()), // Overlaps with B
        ];

        assert!(BenchmarkUtils::validate_no_overlaps(&categories).is_err());
    }

    #[test]
    fn test_validate_overlapping_single_values() {
        let categories = vec![
            BenchmarkCategory::new_single(85, "Good".to_string()),
            BenchmarkCategory::new_single(85, "Excellent".to_string()), // Same value
        ];

        assert!(BenchmarkUtils::validate_no_overlaps(&categories).is_err());
    }

    #[test]
    fn test_validate_non_overlapping() {
        let categories = vec![
            BenchmarkCategory::new_range(70, 79, "B".to_string()),
            BenchmarkCategory::new_range(80, 89, "A".to_string()),
            BenchmarkCategory::new_single(100, "Perfect".to_string()),
        ];

        assert!(BenchmarkUtils::validate_no_overlaps(&categories).is_ok());
    }
}
