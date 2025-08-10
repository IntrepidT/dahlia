use crate::app::components::data_processing::{AssessmentSummary, Progress};
use crate::app::components::student_report::assessments::assessment_card::AssessmentCard;
use crate::app::components::student_report::assessments::expanded_test_list::ExpandedTestList;
use crate::app::models::test::Test;
use leptos::prelude::*;
use leptos::prelude::*;

#[component]
pub fn ProgressOverviewTab(
    assessments: Vec<AssessmentSummary>,
    tests_resource: Resource<Option<Vec<Test>>>,
) -> impl IntoView {
    // State for expanded assessment
    let (expanded_assessment, set_expanded_assessment) = create_signal::<Option<String>>(None);

    // State for filtering
    let (filter_progress, set_filter_progress) = create_signal::<Option<Progress>>(None);
    let (filter_subject, set_filter_subject) = create_signal::<Option<String>>(None);
    let (sort_option, set_sort_option) = create_signal::<String>("name".to_string());

    // Callback for expanding/collapsing assessments
    let on_expand = Callback::new(move |assessment_id: String| {
        if expanded_assessment.get() == Some(assessment_id.clone()) {
            set_expanded_assessment(None);
        } else {
            set_expanded_assessment(Some(assessment_id));
        }
    });

    // Clone assessments for use in multiple closures
    let assessments_clone = assessments.clone();
    let assessments_for_subjects = assessments.clone();

    // Filter and sort assessments
    let filtered_and_sorted_assessments = Memo::new(move |_| {
        let mut filtered_assessments = assessments_clone.clone();

        // Apply progress filter
        if let Some(progress_filter) = filter_progress.get() {
            filtered_assessments = filtered_assessments
                .into_iter()
                .filter(|assessment| assessment.progress == progress_filter)
                .collect();
        }

        // Apply subject filter
        if let Some(subject_filter) = filter_subject.get() {
            if !subject_filter.is_empty() {
                filtered_assessments = filtered_assessments
                    .into_iter()
                    .filter(|assessment| assessment.subject == subject_filter)
                    .collect();
            }
        }

        // Sort assessments
        match sort_option.get().as_str() {
            "name" => {
                filtered_assessments.sort_by(|a, b| a.assessment_name.cmp(&b.assessment_name))
            }
            "progress" => filtered_assessments.sort_by(|a, b| {
                let a_score = if let Some(total) = a.total_possible {
                    a.current_score as f32 / total as f32
                } else {
                    0.0
                };
                let b_score = if let Some(total) = b.total_possible {
                    b.current_score as f32 / total as f32
                } else {
                    0.0
                };
                b_score
                    .partial_cmp(&a_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            "subject" => filtered_assessments.sort_by(|a, b| a.subject.cmp(&b.subject)),
            "test_count" => {
                filtered_assessments.sort_by(|a, b| b.test_details.len().cmp(&a.test_details.len()))
            }
            _ => {}
        }

        filtered_assessments
    });

    // Get unique subjects for filter dropdown
    let unique_subjects = Memo::new(move |_| {
        let mut subjects: Vec<String> = assessments_for_subjects
            .iter()
            .map(|assessment| assessment.subject.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        subjects.sort();
        subjects
    });

    // Calculate summary statistics
    let summary_stats = Memo::new(move |_| {
        let filtered = filtered_and_sorted_assessments.get();
        let total_assessments = filtered.len();
        let completed = filtered
            .iter()
            .filter(|a| a.progress == Progress::Completed)
            .count();
        let in_progress = filtered
            .iter()
            .filter(|a| a.progress == Progress::Ongoing)
            .count();
        let not_started = filtered
            .iter()
            .filter(|a| a.progress == Progress::NotStarted)
            .count();

        let total_tests: usize = filtered.iter().map(|a| a.test_details.len()).sum();

        (
            total_assessments,
            completed,
            in_progress,
            not_started,
            total_tests,
        )
    });

    view! {
        <div class="space-y-6">
            // Header Section
            <div class="flex flex-col space-y-4 sm:flex-row sm:items-center sm:justify-between sm:space-y-0">
                <div>
                    <h2 class="text-2xl font-semibold text-gray-900">Assessment Progress Overview</h2>
                    <p class="mt-1 text-sm text-gray-600">
                        "Track student performance across all assessments"
                    </p>
                </div>
            </div>

            // Summary Statistics Cards
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
                <div class="bg-white rounded-lg border border-gray-200 p-4">
                    <div class="flex items-center">
                        <div class="p-2 bg-blue-100 rounded-lg">
                            <svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                            </svg>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm text-gray-600">Total Assessments</p>
                            <p class="text-2xl font-semibold text-gray-900">{move || summary_stats.get().0}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg border border-gray-200 p-4">
                    <div class="flex items-center">
                        <div class="p-2 bg-green-100 rounded-lg">
                            <svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm text-gray-600">Completed</p>
                            <p class="text-2xl font-semibold text-green-600">{move || summary_stats.get().1}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg border border-gray-200 p-4">
                    <div class="flex items-center">
                        <div class="p-2 bg-yellow-100 rounded-lg">
                            <svg class="w-6 h-6 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm text-gray-600">In Progress</p>
                            <p class="text-2xl font-semibold text-yellow-600">{move || summary_stats.get().2}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg border border-gray-200 p-4">
                    <div class="flex items-center">
                        <div class="p-2 bg-gray-100 rounded-lg">
                            <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm text-gray-600">Not Started</p>
                            <p class="text-2xl font-semibold text-gray-600">{move || summary_stats.get().3}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg border border-gray-200 p-4">
                    <div class="flex items-center">
                        <div class="p-2 bg-purple-100 rounded-lg">
                            <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                            </svg>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm text-gray-600">Total Tests</p>
                            <p class="text-2xl font-semibold text-purple-600">{move || summary_stats.get().4}</p>
                        </div>
                    </div>
                </div>
            </div>

            // Filters and Controls
            <div class="bg-white rounded-lg border border-gray-200 p-4">
                <div class="flex flex-col space-y-4 sm:flex-row sm:items-center sm:space-y-0 sm:space-x-4">
                    // Progress Filter
                    <div class="flex items-center space-x-2">
                        <label class="text-sm font-medium text-gray-700">Progress:</label>
                        <select
                            class="border border-gray-300 rounded-md px-3 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                match value.as_str() {
                                    "completed" => set_filter_progress(Some(Progress::Completed)),
                                    "ongoing" => set_filter_progress(Some(Progress::Ongoing)),
                                    "not_started" => set_filter_progress(Some(Progress::NotStarted)),
                                    _ => set_filter_progress(None),
                                }
                            }
                        >
                            <option value="">"All Progress"</option>
                            <option value="completed">"Completed"</option>
                            <option value="ongoing">"In Progress"</option>
                            <option value="not_started">"Not Started"</option>
                        </select>
                    </div>

                    // Subject Filter
                    <div class="flex items-center space-x-2">
                        <label class="text-sm font-medium text-gray-700">Subject:</label>
                        <select
                            class="border border-gray-300 rounded-md px-3 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                if value.is_empty() {
                                    set_filter_subject(None);
                                } else {
                                    set_filter_subject(Some(value));
                                }
                            }
                        >
                            <option value="">"All Subjects"</option>
                            {move || unique_subjects.get().into_iter().map(|subject| {
                                view! {
                                    <option value={subject.clone()}>{subject.clone()}</option>
                                }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>

                    // Sort Options
                    <div class="flex items-center space-x-2">
                        <label class="text-sm font-medium text-gray-700">Sort by:</label>
                        <select
                            class="border border-gray-300 rounded-md px-3 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                set_sort_option(value);
                            }
                        >
                            <option value="name">"Assessment Name"</option>
                            <option value="progress">"Progress"</option>
                            <option value="subject">"Subject"</option>
                            <option value="test_count">"Test Count"</option>
                        </select>
                    </div>

                    // Clear Filters Button
                    <button
                        class="px-3 py-1 text-sm text-gray-600 hover:text-gray-800 hover:bg-gray-50 rounded-md transition-colors"
                        on:click=move |_| {
                            set_filter_progress(None);
                            set_filter_subject(None);
                            set_sort_option("name".to_string());
                        }
                    >
                        "Clear Filters"
                    </button>
                </div>
            </div>

            // Assessment Cards
            <div class="space-y-4">
                {move || {
                    let filtered_assessments = filtered_and_sorted_assessments.get();

                    if filtered_assessments.is_empty() {
                        view! {
                            <div class="text-center py-12 bg-white rounded-lg border border-gray-200">
                                <div class="w-12 h-12 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center">
                                    <svg class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                    </svg>
                                </div>
                                <h3 class="text-lg font-medium text-gray-900 mb-1">"No assessments found"</h3>
                                <p class="text-gray-500">"Try adjusting your filters or check back later for new assessments."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-6">
                                {filtered_assessments.into_iter().map(|assessment| {
                                    let assessment_id = assessment.assessment_id.clone();
                                    let is_expanded = move || expanded_assessment.get() == Some(assessment_id.clone());

                                    view! {
                                        <div>
                                            <AssessmentCard
                                                assessment=assessment.clone()
                                                tests_resource=tests_resource
                                                on_expand=on_expand
                                                is_expanded=is_expanded()
                                            />

                                            // Expanded Test List
                                            {move || {
                                                if is_expanded() {
                                                    view! {
                                                        <div>
                                                            <ExpandedTestList
                                                                assessment=assessment.clone()
                                                                tests_resource=tests_resource
                                                                show_detailed_test_info=true
                                                            />
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }
                                            }}
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
