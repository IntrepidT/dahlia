use leptos::prelude::*;
use crate::app::models::assessment::Assessment;
use crate::app::models::course::Course;
use crate::app::models::student::GradeEnum;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimpleGradebookFilters {
    // Primary filters - highest impact, always visible
    pub search_term: String,
    pub grade_level: Option<GradeEnum>,
    pub course: Option<String>,
    pub assessment: Option<String>,

    // Secondary filters - show/hide toggle
    pub teacher: Option<i32>,
    pub intervention: bool,
    pub incomplete_only: bool,

    // Sorting and pagination
    pub sort_by: SortBy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortBy {
    Name,
    Grade,
    LastActivity,
    Score,
}

impl Default for SimpleGradebookFilters {
    fn default() -> Self {
        Self {
            search_term: String::new(),
            grade_level: None,
            course: None,
            assessment: None,
            teacher: None,
            intervention: false,
            incomplete_only: false,
            sort_by: SortBy::Name,
        }
    }
}

impl SimpleGradebookFilters {
    pub fn has_active_filters(&self) -> bool {
        !self.search_term.is_empty()
            || self.grade_level.is_some()
            || self.course.is_some()
            || self.assessment.is_some()
            || self.teacher.is_some()
            || self.intervention
            || self.incomplete_only
    }

    pub fn clear_all(&mut self) {
        *self = Self::default();
    }
}

//Main component
#[component]
pub fn GradebookFilters(
    #[prop(into)] filters: Signal<GradebookFilters>,
    #[prop(into)] set_filters: Callback<SimpleGradebookFilters>,
    #[prop(into)] available_teachers: Signal<Vec<(i32, String)>>,
    #[prop(into)] available_assessments: Signal<Vec<Assessment>>,
    #[prop(into)] class_sections: Signal<Vec<String>>,
    #[prop(into)] student_count: Signal<usize>,
) -> impl IntoView {
    let (show_more_filters, set_show_more_filters) = signal(false);

    view! {
        <div class="bg-white border border-gray-200 rounded-md mb-4">
            // Main filter bar - always visible
            <div class="p-4 border-b border-gray-100">
                <div class="flex flex-wrap items-center gap-3">
                    // Search input
                    <div class="flex-1 min-w-64">
                        <div class="relative">
                            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                <svg class="h-4 w-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                                </svg>
                            </div>
                            <input
                                type="text"
                                placeholder="Search students..."
                                class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                prop:value=move || filters.get().search_term
                                on:input=move |ev| {
                                    let mut new_filters = filters.get();
                                    new_filters.search_term = event_target_value(&ev);
                                    set_filters.call(new_filters);
                                }
                            />
                        </div>
                    </div>

                    // Grade filter
                    <div class="min-w-32">
                        <select
                            class="block w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            on:change=move |ev| {
                                let mut new_filters = filters.get();
                                let value = event_target_value(&ev);
                                new_filters.grade_level = if value == "all" {
                                    None
                                } else {
                                    value.parse::<GradeEnum>().ok()
                                };
                                set_filters.call(new_filters);
                            }
                        >
                            <option value="all">"All Grades"</option>
                            <option value="Kindergarten">"Kindergarten"</option>
                            <option value="1st Grade">"1st Grade"</option>
                            <option value="2nd Grade">"2nd Grade"</option>
                            <option value="3rd Grade">"3rd Grade"</option>
                            <option value="4th Grade">"4th Grade"</option>
                            <option value="5th Grade">"5th Grade"</option>
                            <option value="6th Grade">"6th Grade"</option>
                            <option value="7th Grade">"7th Grade"</option>
                            <option value="8th Grade">"8th Grade"</option>
                            <option value="9th Grade">"9th Grade"</option>
                            <option value="10th Grade">"10th Grade"</option>
                            <option value="11th Grade">"11th Grade"</option>
                            <option value="12th Grade">"12th Grade"</option>
                        </select>
                    </div>

                    // Class/Section filter
                    <div class="min-w-40">
                        <select
                            class="block w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            on:change=move |ev| {
                                let mut new_filters = filters.get();
                                let value = event_target_value(&ev);
                                new_filters.course = if value == "all" { None } else { Some(value) };
                                set_filters.call(new_filters);
                            }
                        >
                            <option value="all">"All Classes"</option>
                            {move || {
                                class_sections.get().into_iter().map(|section| {
                                    view! {
                                        <option value={section.clone()}>{section}</option>
                                    }
                                }).collect_view()
                            }}
                        </select>
                    </div>

                    // Assessment filter
                    <div class="min-w-48">
                        <select
                            class="block w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            on:change=move |ev| {
                                let mut new_filters = filters.get();
                                let value = event_target_value(&ev);
                                new_filters.assessment_id = if value == "all" { None } else { Some(value) };
                                set_filters.call(new_filters);
                            }
                        >
                            <option value="all">"All Assessments"</option>
                            {move || {
                                available_assessments.get().into_iter().map(|assessment| {
                                    view! {
                                        <option value={assessment.id.to_string()}>{assessment.name}</option>
                                    }
                                }).collect_view()
                            }}
                        </select>
                    </div>

                    // More filters toggle
                    <button
                        type="button"
                        class="inline-flex items-center px-3 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:ring-2 focus:ring-blue-500"
                        on:click=move |_| set_show_more_filters.update(|show| *show = !*show)
                    >
                        {move || if show_more_filters.get() { "Fewer filters" } else { "More filters" }}
                        <svg class=move || {
                            if show_more_filters.get() {
                                "ml-1 h-4 w-4 transform rotate-180"
                            } else {
                                "ml-1 h-4 w-4"
                            }
                        } fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                        </svg>
                    </button>

                    // Clear all (only show if filters are active)
                    <Show when=move || filters.get().has_active_filters()>
                        <button
                            type="button"
                            class="inline-flex items-center px-3 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                            on:click=move |_| {
                                let mut new_filters = filters.get();
                                new_filters.clear_all();
                                set_filters.call(new_filters);
                            }
                        >
                            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                            "Clear all"
                        </button>
                    </Show>
                </div>
            </div>

            // Additional filters (collapsible)
            <Show when=move || show_more_filters.get()>
                <div class="p-4 bg-gray-50 border-b border-gray-100">
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                        // Teacher filter
                        <div>
                            <label class="block text-xs font-medium text-gray-700 mb-1">"Teacher"</label>
                            <select
                                class="block w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                on:change=move |ev| {
                                    let mut new_filters = filters.get();
                                    let value = event_target_value(&ev);
                                    new_filters.teacher_id = if value == "all" {
                                        None
                                    } else {
                                        value.parse::<i32>().ok()
                                    };
                                    set_filters.call(new_filters);
                                }
                            >
                                <option value="all">"All Teachers"</option>
                                {move || {
                                    available_teachers.get().into_iter().map(|(id, name)| {
                                        view! {
                                            <option value={id.to_string()}>{name}</option>
                                        }
                                    }).collect_view()
                                }}
                            </select>
                        </div>

                        // Simple checkbox filters
                        <div class="space-y-2">
                            <label class="flex items-center">
                                <input
                                    type="checkbox"
                                    class="form-checkbox h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                                    prop:checked=move || filters.get().special_needs
                                    on:change=move |ev| {
                                        let mut new_filters = filters.get();
                                        new_filters.special_needs = event_target_checked(&ev);
                                        set_filters.call(new_filters);
                                    }
                                />
                                <span class="ml-2 text-sm text-gray-700">"Special needs (IEP/504/ESL)"</span>
                            </label>

                            <label class="flex items-center">
                                <input
                                    type="checkbox"
                                    class="form-checkbox h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                                    prop:checked=move || filters.get().intervention
                                    on:change=move |ev| {
                                        let mut new_filters = filters.get();
                                        new_filters.intervention = event_target_checked(&ev);
                                        set_filters.call(new_filters);
                                    }
                                />
                                <span class="ml-2 text-sm text-gray-700">"Receiving intervention"</span>
                            </label>
                        </div>

                        <div class="space-y-2">
                            <label class="flex items-center">
                                <input
                                    type="checkbox"
                                    class="form-checkbox h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                                    prop:checked=move || filters.get().incomplete_only
                                    on:change=move |ev| {
                                        let mut new_filters = filters.get();
                                        new_filters.incomplete_only = event_target_checked(&ev);
                                        set_filters.call(new_filters);
                                    }
                                />
                                <span class="ml-2 text-sm text-gray-700">"Incomplete only"</span>
                            </label>

                            <label class="flex items-center">
                                <input
                                    type="checkbox"
                                    class="form-checkbox h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                                    prop:checked=move || filters.get().at_risk_only
                                    on:change=move |ev| {
                                        let mut new_filters = filters.get();
                                        new_filters.at_risk_only = event_target_checked(&ev);
                                        set_filters.call(new_filters);
                                    }
                                />
                                <span class="ml-2 text-sm text-gray-700">"At risk only"</span>
                            </label>
                        </div>

                        // Sort options
                        <div>
                            <label class="block text-xs font-medium text-gray-700 mb-1">"Sort by"</label>
                            <select
                                class="block w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                on:change=move |ev| {
                                    let mut new_filters = filters.get();
                                    new_filters.sort_by = match event_target_value(&ev).as_str() {
                                        "grade" => SortBy::Grade,
                                        "activity" => SortBy::LastActivity,
                                        "score" => SortBy::Score,
                                        _ => SortBy::Name,
                                    };
                                    set_filters.call(new_filters);
                                }
                            >
                                <option value="name">"Student Name"</option>
                                <option value="grade">"Grade Level"</option>
                                <option value="activity">"Last Activity"</option>
                                <option value="score">"Average Score"</option>
                            </select>
                        </div>
                    </div>
                </div>
            </Show>

            // Results summary
            <div class="px-4 py-2 bg-gray-50 border-b border-gray-100 text-xs text-gray-600">
                {move || {
                    let count = student_count.get();
                    let filter_count = if filters.get().has_active_filters() {
                        format!(" (filtered)")
                    } else {
                        String::new()
                    };
                    format!("Showing {} students{}", count, filter_count)
                }}
            </div>
        </div>
    }
}

// ============================================================================
// Active Filters Summary - Shows applied filters as removable pills
// ============================================================================

#[component]
pub fn ActiveFilterSummary(
    #[prop(into)] filters: Signal<SimpleGradebookFilters>,
    #[prop(into)] set_filters: Callback<SimpleGradebookFilters>,
) -> impl IntoView {
    view! {
        <Show when=move || filters.get().has_active_filters()>
            <div class="flex flex-wrap gap-2 mb-3">
                // Search term
                <Show when=move || !filters.get().search_term.is_empty()>
                    <FilterPill
                        label=move || format!("Search: \"{}\"", filters.get().search_term)
                        on_remove=move || {
                            let mut new_filters = filters.get();
                            new_filters.search_term.clear();
                            set_filters.call(new_filters);
                        }
                    />
                </Show>

                // Grade level
                <Show when=move || filters.get().grade_level.is_some()>
                    <FilterPill
                        label=move || format!("Grade: {}", filters.get().grade_level.unwrap().to_string())
                        on_remove=move || {
                            let mut new_filters = filters.get();
                            new_filters.grade_level = None;
                            set_filters.call(new_filters);
                        }
                    />
                </Show>

                // Class section
                <Show when=move || filters.get().class_section.is_some()>
                    <FilterPill
                        label=move || format!("Class: {}", filters.get().class_section.as_ref().unwrap())
                        on_remove=move || {
                            let mut new_filters = filters.get();
                            new_filters.class_section = None;
                            set_filters.call(new_filters);
                        }
                    />
                </Show>

                // Boolean filters
                <Show when=move || filters.get().special_needs>
                    <FilterPill
                        label="Special Needs"
                        on_remove=move || {
                            let mut new_filters = filters.get();
                            new_filters.special_needs = false;
                            set_filters.call(new_filters);
                        }
                    />
                </Show>

                <Show when=move || filters.get().intervention>
                    <FilterPill
                        label="Intervention"
                        on_remove=move || {
                            let mut new_filters = filters.get();
                            new_filters.intervention = false;
                            set_filters.call(new_filters);
                        }
                    />
                </Show>

                <Show when=move || filters.get().incomplete_only>
                    <FilterPill
                        label="Incomplete Only"
                        on_remove=move || {
                            let mut new_filters = filters.get();
                            new_filters.incomplete_only = false;
                            set_filters.call(new_filters);
                        }
                    />
                </Show>

                <Show when=move || filters.get().at_risk_only>
                    <FilterPill
                        label="At Risk Only"
                        on_remove=move || {
                            let mut new_filters = filters.get();
                            new_filters.at_risk_only = false;
                            set_filters.call(new_filters);
                        }
                    />
                </Show>
            </div>
        </Show>
    }
}

// ============================================================================
// Helper Components
// ============================================================================

#[component]
fn FilterPill(
    #[prop(into)] label: Signal<String>,
    #[prop(into)] on_remove: Callback<()>,
) -> impl IntoView {
    view! {
        <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
            {move || label.get()}
            <button
                type="button"
                class="ml-1 inline-flex items-center p-0.5 rounded-full text-blue-600 hover:bg-blue-200 hover:text-blue-800 focus:outline-none"
                on:click=move |_| on_remove.call(())
            >
                <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"/>
                </svg>
            </button>
        </span>
    }
}

// ============================================================================
// Filter Logic Functions (for server-side filtering)
// ============================================================================

impl SimpleGradebookFilters {
    /// Generate SQL WHERE clause based on active filters
    pub fn to_sql_conditions(&self) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        if !self.search_term.is_empty() {
            conditions.push(
                "(firstname ILIKE $? OR lastname ILIKE $? OR student_id=:text ILIKE $?)"
                    .to_string(),
            );
            let search_param = format!("%{}%", self.search_term);
            params.push(search_param.clone());
            params.push(search_param.clone());
            params.push(search_param);
        }

        if let Some(grade) = &self.grade_level {
            conditions.push("grade_level = $?".to_string());
            params.push(grade.to_string());
        }

        if let Some(class) = &self.class_section {
            conditions.push("class_section = $?".to_string());
            params.push(class.clone());
        }

        if let Some(teacher_id) = self.teacher_id {
            conditions.push("teacher_id = $?".to_string());
            params.push(teacher_id.to_string());
        }

        if self.special_needs {
            conditions.push("(iep = true OR plan_504 = true OR esl = true)".to_string());
        }

        if self.intervention {
            conditions.push(
                "intervention_status IS NOT NULL AND intervention_status != 'None'".to_string(),
            );
        }

        if self.incomplete_only {
            conditions.push("assessment_progress < 100".to_string());
        }

        if self.at_risk_only {
            conditions.push("risk_level = 'High'".to_string());
        }

        let where_clause = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        };

        (where_clause, params)
    }

    /// Get ORDER BY clause based on sort option
    pub fn to_sql_order(&self) -> String {
        match self.sort_by {
            SortBy::Name => "lastname, firstname".to_string(),
            SortBy::Grade => "grade_level, lastname, firstname".to_string(),
            SortBy::LastActivity => {
                "last_assessment_date DESC NULLS LAST, lastname, firstname".to_string()
            }
            SortBy::Score => "average_score DESC NULLS LAST, lastname, firstname".to_string(),
        }
    }
}
