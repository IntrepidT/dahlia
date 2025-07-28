use crate::app::models::test::BenchmarkCategory;

pub struct ColorUtils;

impl ColorUtils {
    /// Converts hex colors from benchmark categories to Tailwind CSS classes
    pub fn hex_to_tailwind_classes(hex_color: &str) -> String {
        match hex_color.to_lowercase().as_str() {
            "#10b981" => "bg-green-100 text-green-800".to_string(), // green
            "#06b6d4" => "bg-cyan-100 text-cyan-800".to_string(),   // cyan
            "#f59e0b" => "bg-amber-100 text-amber-800".to_string(), // amber
            "#ef4444" => "bg-red-100 text-red-800".to_string(),     // red
            "#8b5cf6" => "bg-purple-100 text-purple-800".to_string(), // purple
            "#f43f5e" => "bg-rose-100 text-rose-800".to_string(),   // rose
            "#6b7280" => "bg-gray-100 text-gray-800".to_string(),   // gray
            _ => "bg-gray-100 text-gray-800".to_string(),           // default fallback
        }
    }

    /// Gets badge color classes based on score percentage and benchmark categories
    pub fn get_badge_classes_for_score(
        score: i32,
        max_score: i32,
        benchmark_categories: Option<&Vec<BenchmarkCategory>>,
    ) -> String {
        if max_score <= 0 {
            return "bg-gray-100 text-gray-800".to_string();
        }

        let percentage = (score as f64 / max_score as f64) * 100.0;

        // Check custom benchmark categories first
        if let Some(categories) = benchmark_categories {
            for category in categories {
                // Determine if benchmark category uses percentage or raw score
                let (min_check, max_check) = if Self::is_percentage_based(category, max_score) {
                    // Category uses percentages (0-100 range)
                    (category.min as f64, category.max as f64)
                } else {
                    // Category uses raw scores - convert to percentage for comparison
                    let min_percent = (category.min as f64 / max_score as f64) * 100.0;
                    let max_percent = (category.max as f64 / max_score as f64) * 100.0;
                    (min_percent, max_percent)
                };

                if percentage >= min_check && percentage <= max_check {
                    return Self::hex_to_tailwind_classes(&category.get_color());
                }
            }
        }

        // Fallback color scheme
        Self::get_default_color_for_percentage(percentage)
    }

    /// Determines if a benchmark category is using percentage (0-100) or raw score values
    fn is_percentage_based(category: &BenchmarkCategory, max_score: i32) -> bool {
        // If the category max is <= 100 and the test max_score is > 100,
        // assume category is percentage-based
        // This is a heuristic - you might want to add a field to BenchmarkCategory to be explicit
        category.max <= 100 && max_score > 100
    }

    /// Alternative method that uses raw score comparison (if you know categories use raw scores)
    pub fn get_badge_classes_for_raw_score(
        score: i32,
        max_score: i32,
        benchmark_categories: Option<&Vec<BenchmarkCategory>>,
    ) -> String {
        if max_score <= 0 {
            return "bg-gray-100 text-gray-800".to_string();
        }

        // Check custom benchmark categories using raw score values
        if let Some(categories) = benchmark_categories {
            for category in categories {
                if score >= category.min && score <= category.max {
                    return Self::hex_to_tailwind_classes(&category.get_color());
                }
            }
        }

        // Fallback to percentage-based colors
        let percentage = (score as f64 / max_score as f64) * 100.0;
        Self::get_default_color_for_percentage(percentage)
    }

    /// Alternative method that uses percentage comparison (if you know categories use percentages)
    pub fn get_badge_classes_for_percentage(
        score: i32,
        max_score: i32,
        benchmark_categories: Option<&Vec<BenchmarkCategory>>,
    ) -> String {
        if max_score <= 0 {
            return "bg-gray-100 text-gray-800".to_string();
        }

        let percentage = (score as f64 / max_score as f64) * 100.0;

        // Check custom benchmark categories using percentage values
        if let Some(categories) = benchmark_categories {
            for category in categories {
                let min_percent = category.min as f64;
                let max_percent = category.max as f64;
                if percentage >= min_percent && percentage <= max_percent {
                    return Self::hex_to_tailwind_classes(&category.get_color());
                }
            }
        }

        // Fallback color scheme
        Self::get_default_color_for_percentage(percentage)
    }

    /// Gets default color classes based on percentage (when no custom categories)
    fn get_default_color_for_percentage(percentage: f64) -> String {
        if percentage >= 90.0 {
            "bg-green-100 text-green-800".to_string() // Excellent
        } else if percentage >= 80.0 {
            "bg-cyan-100 text-cyan-800".to_string() // Good
        } else if percentage >= 70.0 {
            "bg-amber-100 text-amber-800".to_string() // Satisfactory
        } else if percentage >= 60.0 {
            "bg-rose-100 text-rose-800".to_string() // Needs Improvement
        } else {
            "bg-red-100 text-red-800".to_string() // Poor
        }
    }

    /// Converts hex colors to inline style string (alternative to Tailwind)
    pub fn hex_to_inline_style(hex_color: &str) -> String {
        let (bg_light, text_color) = Self::get_color_variants(hex_color);
        format!(
            "background-color: {}; color: {}; border: 1px solid {};",
            bg_light, text_color, hex_color
        )
    }

    /// Gets light background and contrasting text color for a given hex color
    fn get_color_variants(hex_color: &str) -> (&'static str, &'static str) {
        match hex_color.to_lowercase().as_str() {
            "#10b981" => ("#d1fae5", "#065f46"), // green light background, dark text
            "#06b6d4" => ("#cffafe", "#164e63"), // cyan light background, dark text
            "#f59e0b" => ("#fef3c7", "#92400e"), // amber light background, dark text
            "#ef4444" => ("#fee2e2", "#991b1b"), // red light background, dark text
            "#8b5cf6" => ("#ede9fe", "#581c87"), // purple light background, dark text
            "#f43f5e" => ("#ffe4e6", "#9f1239"), // rose light background, dark text
            "#6b7280" => ("#f3f4f6", "#374151"), // gray light background, dark text
            _ => ("#f3f4f6", "#374151"),         // default gray
        }
    }

    /// Gets inline style for score badges based on benchmark categories
    pub fn get_badge_style_for_score(
        score: i32,
        max_score: i32,
        benchmark_categories: Option<&Vec<BenchmarkCategory>>,
    ) -> String {
        if max_score <= 0 {
            return "background-color: #f3f4f6; color: #374151; border: 1px solid #d1d5db;"
                .to_string();
        }

        let percentage = (score as f64 / max_score as f64) * 100.0;

        // Check custom benchmark categories first
        if let Some(categories) = benchmark_categories {
            for category in categories {
                let (min_check, max_check) = if Self::is_percentage_based(category, max_score) {
                    (category.min as f64, category.max as f64)
                } else {
                    let min_percent = (category.min as f64 / max_score as f64) * 100.0;
                    let max_percent = (category.max as f64 / max_score as f64) * 100.0;
                    (min_percent, max_percent)
                };

                if percentage >= min_check && percentage <= max_check {
                    return Self::hex_to_inline_style(&category.get_color());
                }
            }
        }

        // Fallback using default colors
        let hex_color = if percentage >= 90.0 {
            "#10b981" // green
        } else if percentage >= 80.0 {
            "#06b6d4" // cyan
        } else if percentage >= 70.0 {
            "#f59e0b" // amber
        } else if percentage >= 60.0 {
            "#f43f5e" // rose
        } else {
            "#ef4444" // red
        };

        Self::hex_to_inline_style(hex_color)
    }
}
