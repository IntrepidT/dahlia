use leptos::prelude::*;
pub mod student_charts;
pub mod student_results_summary;
pub mod test_pie_chart;

// Re-export commonly used types and components
pub use student_results_summary::{
    AssessmentSummary, Progress, StudentResultsSummary, TestDetail, TestHistoryEntry,
};

// Only export server functions when SSR feature is enabled
#[cfg(feature = "ssr")]
pub use student_results_summary::get_student_results;

// Always export chart components (they handle their own feature gating internally)
pub use student_charts::{
    AssessmentProgressChart, AssessmentRadarChart, PerformanceDistributionChart,
    TestAreaPerformanceChart, TestScoresTimelineChart,
};
pub use test_pie_chart::*;
