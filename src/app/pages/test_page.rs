use crate::app::components::auth::server_auth_components::ServerAuthGuard;
use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::test_components::select_test_modal::SelectTestModal;
use crate::app::components::{Header, MathTestDisplay, Toast, ToastMessage, ToastMessageType};
use crate::app::models::test::CreateNewTestRequest;
use crate::app::models::{DeleteTestRequest, Test, TestType};
use crate::app::server_functions::questions::duplicate_and_randomize_questions;
use crate::app::server_functions::{
    get_tests,
    tests::{add_test, delete_test},
};
use leptos::callback::*;
use leptos::*;
use std::rc::Rc;

// =============================================================================
// TYPES AND ENUMS
// =============================================================================

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TestFilter {
    All,
    Math,
    Reading,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Cards,   // Your existing card view
    Table,   // New dense table view
    Compact, // Ultra-compact list
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestGroup {
    pub base_test: Test,
    pub variations: Vec<Test>,
    pub is_expanded: bool,
}

// =============================================================================
// STYLES CONSTANTS
// =============================================================================

mod styles {
    pub const PRIMARY_BUTTON: &str = "bg-[#2E3A59] px-4 py-2 rounded-md text-white font-medium text-sm shadow-sm hover:bg-[#1f2937] transition-all focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:ring-offset-2";
    pub const SECONDARY_BUTTON: &str = "bg-white px-4 py-2 rounded-md text-gray-700 font-medium text-sm border border-gray-300 shadow-sm hover:bg-gray-50 transition-all focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2";
    pub const SECONDARY_BUTTON_ACTIVE: &str = "bg-indigo-100 px-4 py-2 rounded-md text-indigo-700 font-medium text-sm border border-[#2E3A59] shadow-sm hover:bg-indigo-50 transition-all focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:ring-offset-2";
    pub const DANGER_BUTTON: &str = "bg-white px-4 py-2 rounded-md text-red-600 font-medium text-sm border border-gray-300 shadow-sm hover:bg-red-50 transition-all focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2";
    pub const FILTER_TAB: &str = "px-4 py-2 text-sm font-medium rounded-lg transition-all focus:outline-none focus:ring-2 focus:ring-offset-2";
    pub const SEARCH_INPUT: &str = "focus:ring-indigo-500 focus:border-indigo-500 block w-full pl-10 pr-3 text-sm border-gray-300 rounded-md h-10 border";
    pub const MODAL_BACKDROP: &str =
        "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4";
    pub const MODAL_CONTAINER: &str = "bg-white rounded-xl shadow-2xl max-w-md w-full";
    pub const CARD: &str = "bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden hover:shadow-md transition-all duration-200";
    pub const TABLE_CONTAINER: &str =
        "bg-white rounded-lg border border-gray-200 overflow-hidden shadow-sm";
    pub const TABLE_HEADER: &str = "bg-gray-50 border-b border-gray-200";
    pub const TABLE_ROW: &str = "border-b border-gray-100 hover:bg-gray-50 transition-colors";
    pub const TABLE_ROW_DELETE: &str =
        "border-b border-gray-100 hover:bg-red-50 bg-red-25 transition-colors";
    pub const TABLE_CELL: &str = "px-3 py-2 text-sm";
    pub const TABLE_HEADER_CELL: &str =
        "px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider";
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

impl TestGroup {
    pub fn new(base_test: Test) -> Self {
        Self {
            base_test,
            variations: Vec::new(),
            is_expanded: false,
        }
    }

    pub fn add_variation(&mut self, variation: Test) {
        self.variations.push(variation);
    }

    pub fn total_tests(&self) -> usize {
        1 + self.variations.len()
    }

    pub fn has_variations(&self) -> bool {
        !self.variations.is_empty()
    }

    pub fn get_variation_types(&self) -> Vec<String> {
        self.variations
            .iter()
            .filter_map(|v| {
                if v.name.contains(" - ") {
                    v.name.split(" - ").nth(1).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}

fn get_test_type_styling(test_type: &TestType) -> (&'static str, &'static str) {
    match test_type {
        TestType::Math => ("bg-green-100 text-green-800 border-green-200", "Math"),
        TestType::Reading => ("bg-purple-100 text-purple-800 border-purple-200", "Reading"),
        TestType::PhonemicAwareness => (
            "bg-yellow-100 text-yellow-800 border-yellow-200",
            "Phonemic Awareness",
        ),
        TestType::Spelling => (
            "bg-orange-100 text-orange-800 border-orange-200",
            "Spelling",
        ),
        TestType::Other => ("bg-gray-100 text-gray-800 border-gray-200", "Other"),
    }
}

fn get_variation_styling(variation_name: &str) -> (&'static str, &'static str) {
    let name_lower = variation_name.to_lowercase();
    match name_lower.as_str() {
        name if name.contains("randomized") => {
            ("bg-blue-100 text-blue-800 border-blue-300", "Randomized")
        }
        name if name.contains("distinct") => {
            ("bg-green-100 text-green-800 border-green-300", "Distinct")
        }
        name if name.contains("practice") => (
            "bg-purple-100 text-purple-800 border-purple-300",
            "Practice",
        ),
        _ => ("bg-gray-100 text-gray-800 border-gray-300", "Standard"),
    }
}

fn is_variation_test(test: &Test) -> bool {
    test.name.contains(" - ")
        && (test.name.to_lowercase().contains("randomized")
            || test.name.to_lowercase().contains("distinct")
            || test.name.to_lowercase().contains("practice")
            || test.comments.to_lowercase().contains("variation:"))
}

async fn get_next_variant_number_for_base(
    base_test_name: &str,
) -> Result<i32, leptos::ServerFnError> {
    let all_tests = get_tests().await?;

    // Find all tests related to this base test
    let related_tests: Vec<&Test> = all_tests
        .iter()
        .filter(|test| {
            let test_base_name = if test.name.contains(" - ") {
                test.name.split(" - ").next().unwrap_or(&test.name)
            } else {
                &test.name
            };
            test_base_name == base_test_name
        })
        .collect();

    // Find the highest variant number
    let max_variant = related_tests
        .iter()
        .map(|test| test.test_variant)
        .max()
        .unwrap_or(0);

    Ok(max_variant + 1)
}

// =============================================================================
// COMPONENT FRAGMENTS
// =============================================================================

#[component]
fn StatsPanel(
    test_stats: Memo<(usize, usize, usize, usize, usize)>,
    show_stats: ReadSignal<bool>,
    set_show_stats: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        {move || {
            if show_stats() {
                let (total, math, reading, other, variations) = test_stats();
                view! {
                    <div class="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg p-4 border border-blue-200">
                        <div class="flex items-center justify-between mb-3">
                            <h3 class="text-lg font-semibold text-gray-800">Test Collection Overview</h3>
                            <button
                                class="text-gray-400 hover:text-gray-600 transition-colors"
                                on:click=move |_| set_show_stats(false)
                            >
                                "Hide"
                            </button>
                        </div>
                        <div class="grid grid-cols-2 sm:grid-cols-5 gap-4">
                            <StatCard value=total label="Total Tests" color="blue" />
                            <StatCard value=math label="Math Tests" color="green" />
                            <StatCard value=reading label="Reading Tests" color="purple" />
                            <StatCard value=other label="Other Tests" color="gray" />
                            <StatCard value=variations label="Variations" color="orange" />
                        </div>
                    </div>
                }
            } else {
                view! {
                    <div>
                        <button
                            class="flex items-center space-x-2 text-blue-600 hover:text-blue-800 transition-colors"
                            on:click=move |_| set_show_stats(true)
                        >
                            <span class="text-sm font-medium">Show Statistics</span>
                        </button>
                    </div>
                }
            }
        }}
    }
}

#[component]
fn StatCard(value: usize, label: &'static str, color: &'static str) -> impl IntoView {
    let color_class = match color {
        "blue" => "text-blue-600",
        "green" => "text-green-600",
        "purple" => "text-purple-600",
        "gray" => "text-gray-600",
        "orange" => "text-orange-600",
        _ => "text-gray-600",
    };

    view! {
        <div class="text-center">
            <div class=format!("text-2xl font-bold {}", color_class)>{value}</div>
            <div class="text-xs text-gray-600">{label}</div>
        </div>
    }
}

#[component]
fn FilterTabs(
    test_filter: ReadSignal<TestFilter>,
    set_test_filter: WriteSignal<TestFilter>,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap gap-2">
            <FilterTab
                filter=TestFilter::All
                label="All Tests"
                current_filter=test_filter
                set_filter=set_test_filter
                active_color="blue"
            />
            <FilterTab
                filter=TestFilter::Math
                label="Math Only"
                current_filter=test_filter
                set_filter=set_test_filter
                active_color="green"
            />
            <FilterTab
                filter=TestFilter::Reading
                label="Reading Only"
                current_filter=test_filter
                set_filter=set_test_filter
                active_color="purple"
            />
            <FilterTab
                filter=TestFilter::Other
                label="Other"
                current_filter=test_filter
                set_filter=set_test_filter
                active_color="gray"
            />
        </div>
    }
}

#[component]
fn FilterTab(
    filter: TestFilter,
    label: &'static str,
    current_filter: ReadSignal<TestFilter>,
    set_filter: WriteSignal<TestFilter>,
    active_color: &'static str,
) -> impl IntoView {
    let active_class = match active_color {
        "blue" => "bg-blue-100 text-blue-700 border border-blue-300",
        "green" => "bg-green-100 text-green-700 border border-green-300",
        "purple" => "bg-purple-100 text-purple-700 border border-purple-300",
        "gray" => "bg-gray-100 text-gray-700 border border-gray-300",
        _ => "bg-blue-100 text-blue-700 border border-blue-300",
    };

    view! {
        <button
            class=move || {
                let base_style = styles::FILTER_TAB;
                if current_filter() == filter {
                    format!("{} {}", base_style, active_class)
                } else {
                    format!("{} bg-white text-gray-600 border border-gray-300 hover:bg-gray-50", base_style)
                }
            }
            on:click=move |_| set_filter(filter)
        >
            <span>{label}</span>
        </button>
    }
}

#[component]
fn SearchBar(
    search_term: ReadSignal<String>,
    set_search_term: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div class="relative flex-1 max-w-md">
            <input
                type="text"
                class=styles::SEARCH_INPUT
                placeholder="Search tests and variations..."
                prop:value=move || search_term()
                on:input=move |ev| set_search_term(event_target_value(&ev))
            />
        </div>
    }
}

#[component]
fn ActionButtons(
    if_show_edit: ReadSignal<bool>,
    if_show_delete: ReadSignal<bool>,
    view_mode: ReadSignal<ViewMode>,
    set_view_mode: WriteSignal<ViewMode>,
    on_click_add: impl Fn(ev::MouseEvent) + 'static + Clone,
    on_click_delete_mode: impl Fn(ev::MouseEvent) + 'static + Clone,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 items-center w-full gap-3">

            // Center section - view mode toggle on desktop
            <div class="sm:flex justify-end mr-10">
                <div class="flex items-center space-x-1 bg-gray-100 rounded-lg p-1">
                    <button
                        class=move || {
                            if view_mode() == ViewMode::Cards {
                                "px-3 py-1.5 text-sm font-medium bg-white text-gray-900 rounded-md shadow-sm"
                            } else {
                                "px-3 py-1.5 text-sm font-medium text-gray-600 hover:text-gray-900 transition-colors"
                            }
                        }
                        on:click=move |_| set_view_mode(ViewMode::Cards)
                    >
                        "Cards"
                    </button>

                    <button
                        class=move || {
                            if view_mode() == ViewMode::Table {
                                "px-3 py-1.5 text-sm font-medium bg-white text-gray-900 rounded-md shadow-sm"
                            } else {
                                "px-3 py-1.5 text-sm font-medium text-gray-600 hover:text-gray-900 transition-colors"
                            }
                        }
                        on:click=move |_| set_view_mode(ViewMode::Table)
                    >
                        "Table"
                    </button>

                    <button
                        class=move || {
                            if view_mode() == ViewMode::Compact {
                                "px-3 py-1.5 text-sm font-medium bg-white text-gray-900 rounded-md shadow-sm"
                            } else {
                                "px-3 py-1.5 text-sm font-medium text-gray-600 hover:text-gray-900 transition-colors"
                            }
                        }
                        on:click=move |_| set_view_mode(ViewMode::Compact)
                    >
                        "Compact"
                    </button>
                </div>
            </div>

            // Right section - action buttons
            <div class="flex space-x-2 justify-end col-span-2 sm:col-span-1">
                <button on:click=on_click_add class=styles::PRIMARY_BUTTON>
                    "New Test"
                </button>

                <button
                    class="px-4 py-2 bg-purple-600 text-white rounded-md font-medium text-sm shadow-sm hover:bg-purple-700 transition-all focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2"
                    on:click=move |_| {
                        let navigate = leptos_router::use_navigate();
                        navigate("/test-variations", Default::default());
                    }
                >
                    "Manage Variations"
                </button>

                <button
                    on:click=on_click_delete_mode
                    class=move || {
                        if if_show_delete() {
                            "px-4 py-2 bg-red-600 text-white rounded-md font-medium text-sm shadow-sm hover:bg-red-700 transition-all focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2"
                        } else {
                            styles::DANGER_BUTTON
                        }
                    }
                    title=move || {
                        if if_show_delete() {
                            "Exit Delete Mode - Click to stop deleting"
                        } else {
                            "Enter Delete Mode - Show delete buttons"
                        }
                    }
                >
                    {move || {
                        if if_show_delete() {
                            "🗑️ Exit Delete Mode"
                        } else {
                            "Delete Mode"
                        }
                    }}
                </button>
            </div>
        </div>
    }
}

#[component]
fn EmptyState(
    test_filter: ReadSignal<TestFilter>,
    on_click_add: impl Fn(ev::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div class="text-center py-12">
            <h3 class="mt-4 text-lg font-medium text-gray-900">No tests found</h3>
            <p class="mt-2 text-sm text-gray-500">
                {move || match test_filter() {
                    TestFilter::All => "Get started by creating your first test.",
                    TestFilter::Math => "No math tests found. Create a new math test to get started.",
                    TestFilter::Reading => "No reading tests found. Create a new reading test to get started.",
                    TestFilter::Other => "No other tests found. Create a new test to get started.",
                }}
            </p>
            <div class="mt-6">
                <button on:click=on_click_add class=styles::PRIMARY_BUTTON>
                    "Create Your First Test"
                </button>
            </div>
        </div>
    }
}

#[component]
fn TestGroupCard(
    group: TestGroup,
    expanded_groups: ReadSignal<std::collections::HashSet<String>>,
    if_show_delete: ReadSignal<bool>,
    on_delete_test: Callback<String>,
    on_create_variation: Callback<Test>,
    toggle_group_expansion: impl Fn(String) + 'static + Clone,
) -> impl IntoView {
    let base_test = group.base_test.clone();
    let variations = group.variations.clone();
    let base_name = base_test.name.clone();
    let base_name_for_memo = base_name.clone();
    let has_variations = group.has_variations();

    let is_expanded = create_memo(move |_| expanded_groups().contains(&base_name_for_memo));

    let (test_type_badge_class, test_type_label) = get_test_type_styling(&base_test.testarea);

    view! {
        <div class=move || {
            let base_class = "bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden hover:shadow-md transition-all duration-200";
            if if_show_delete() {
                format!("{} ring-2 ring-red-200", base_class)
            } else {
                base_class.to_string()
            }
        }>
            // Base test header
            <div class=move || {
                let base_class = "bg-gradient-to-r from-gray-50 to-gray-100 px-6 py-5 border-b border-gray-200";
                if if_show_delete() {
                    format!("{} bg-red-50 border-red-200", base_class)
                } else {
                    base_class.to_string()
                }
            }>
                <div class="flex items-start justify-between">
                    <div class="flex-1">
                        <div class="flex items-center space-x-3 mb-2">
                            <h3 class="text-xl font-semibold text-gray-900">{base_test.name.clone()}</h3>
                            <span class=format!("inline-flex items-center px-3 py-1 rounded-full text-xs font-medium border {}", test_type_badge_class)>
                                {test_type_label}
                            </span>
                            // Delete mode indicator
                            {move || {
                                if if_show_delete() {
                                    view! {
                                        <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800 border border-red-300">
                                            "🗑️ Delete Mode"
                                        </span>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }
                            }}
                        </div>

                        <div class="flex flex-wrap items-center gap-4 text-sm text-gray-600">
                            <div class="flex items-center space-x-1">
                                <span class="font-medium">{base_test.score} points</span>
                            </div>
                            <div class="flex items-center space-x-1">
                                <span>Grade: {base_test.grade_level.as_ref().map(|g| format!("{:?}", g)).unwrap_or("Not specified".to_string())}</span>
                            </div>
                            {if has_variations {
                                view! {
                                    <div class="flex items-center space-x-1 text-blue-600">
                                        <span>Test Variations: {variations.len()}</span>
                                    </div>
                                }
                            } else {
                                view! { <div></div> }
                            }}
                        </div>
                    </div>

                    <div class="flex items-center space-x-2">
                        <TestActionButtons test=base_test.clone() on_create_variation=Some(on_create_variation)/>
                        {if has_variations {
                            let toggle_fn = toggle_group_expansion.clone();
                            let base_name_clone = base_name.clone();
                            view! {
                                <button
                                    class="px-3 py-1 text-sm text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded-md transition-all duration-200 border border-gray-300"
                                    on:click=move |_| {
                                        log::info!("Toggling expansion for: {}", base_name_clone.clone());
                                        toggle_fn(base_name_clone.clone());
                                    }
                                >
                                    {move || {
                                        if is_expanded() {
                                            "▼ Hide Variations"
                                        } else {
                                            "▶ Show Variations"
                                        }
                                    }}
                                </button>
                            }.into_view()
                        } else {
                            view! { <span></span> }.into_view()
                        }}
                    </div>
                </div>
            </div>

            // Variations section
            {if has_variations {
                view! {
                    <div class=move || {
                        if is_expanded() {
                            let base_class = "p-6 bg-gray-50 border-t border-gray-200";
                            if if_show_delete() {
                                format!("{} bg-red-25", base_class)
                            } else {
                                base_class.to_string()
                            }
                        } else {
                            "hidden".to_string()
                        }
                    }>
                        <div class="flex items-center justify-between mb-4">
                            <h4 class="text-lg font-medium text-gray-900">
                                "Test Variations (" {variations.len()} ")"
                            </h4>
                            {move || {
                                if if_show_delete() {
                                    view! {
                                        <div class="text-sm text-red-600 font-medium">
                                            "⚠️ Click variation delete buttons below"
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }
                            }}
                        </div>
                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                            <For
                                each=move || variations.clone()
                                key=|variation| variation.test_id.clone()
                                children=move |variation: Test| {
                                    view! {
                                        <VariationCard
                                            variation=variation
                                            if_show_delete=if_show_delete
                                            on_delete_test=on_delete_test
                                        />
                                    }
                                }
                            />
                        </div>
                    </div>
                }.into_view()
            } else {
                view! { <div class="hidden"></div> }.into_view()
            }}
        </div>
    }
}

#[component]
fn CompactListView(
    all_tests: Vec<Test>,
    if_show_delete: ReadSignal<bool>,
    on_delete_test: Callback<String>,
    on_create_variation: Callback<Test>,
) -> impl IntoView {
    // Group tests by base name for compact display
    let grouped_tests = create_memo(move |_| {
        let tests = all_tests.clone();
        let mut groups: std::collections::HashMap<String, (Test, Vec<Test>)> =
            std::collections::HashMap::new();

        for test in tests {
            let base_name = if test.name.contains(" - ") {
                test.name
                    .split(" - ")
                    .next()
                    .unwrap_or(&test.name)
                    .to_string()
            } else {
                test.name.clone()
            };

            if is_variation_test(&test) {
                groups
                    .entry(base_name)
                    .and_modify(|(_, variations)| variations.push(test.clone()))
                    .or_insert((test.clone(), vec![test.clone()])); // fallback if no base test
            } else {
                groups
                    .entry(base_name.clone())
                    .and_modify(|(base, _)| *base = test.clone())
                    .or_insert((test.clone(), Vec::new()));
            }
        }

        let mut sorted_groups: Vec<(Test, Vec<Test>)> = groups.into_values().collect();
        sorted_groups.sort_by(|a, b| a.0.name.cmp(&b.0.name));
        sorted_groups
    });

    view! {
    <div class="space-y-2">
        <For
            each=move || grouped_tests()
            key=|(base_test, _)| base_test.test_id.clone()
            children=move |(base_test, variations): (Test, Vec<Test>)| {
                let (type_class, type_label) = get_test_type_styling(&base_test.testarea);
                let variation_count = variations.len();

                view! {
                    <div class=move || {
                        let base_class = "bg-white border border-gray-200 rounded-lg overflow-hidden hover:shadow-sm transition-all";
                        if if_show_delete() {
                            format!("{} ring-2 ring-red-200", base_class)
                        } else {
                            base_class.to_string()
                        }
                    }>
                        // Base test row
                        <div class="flex items-center justify-between p-4 border-b border-gray-100 last:border-b-0">
                            <div class="flex items-center space-x-3 flex-1 min-w-0">
                                <span class=format!("inline-flex px-2 py-1 text-xs font-medium rounded {}", type_class)>
                                    {type_label}
                                </span>
                                <span class="font-medium text-gray-900 truncate" title=base_test.name.clone()>
                                    {base_test.name.clone()}
                                </span>
                                <span class="text-sm text-gray-500 font-mono">{base_test.score}"pts"</span>
                                {if variation_count > 0 {
                                    view! {
                                        <span class="inline-flex px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded">
                                            {variation_count}" var"
                                        </span>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }}
                                {move || {
                                    if if_show_delete() {
                                        view! {
                                            <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800 border border-red-300">
                                                "🗑️ Delete Mode"
                                            </span>
                                        }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }
                                }}
                            </div>

                            <CompactActions
                                test=base_test.clone()
                                is_variation=false
                                if_show_delete=if_show_delete
                                on_delete_test=on_delete_test.clone()
                                on_create_variation=Some(on_create_variation.clone())
                            />
                        </div>

                        // Variations (if any)
                        {if !variations.is_empty() {
                            view! {
                                <div class=move || {
                                    if if_show_delete() {
                                        "bg-red-25"
                                    } else {
                                        "bg-gray-50"
                                    }
                                }>
                                    <For
                                        each=move || variations.clone()
                                        key=|var| var.test_id.clone()
                                        children=move |variation: Test| {
                                            let var_name = if variation.name.contains(" - ") {
                                                variation.name.split(" - ").nth(1).unwrap_or("Variation").to_string()
                                            } else {
                                                "Variation".to_string()
                                            };

                                            view! {
                                                <div class="flex items-center justify-between px-6 py-3 border-b border-gray-200 last:border-b-0">
                                                    <div class="flex items-center space-x-3">
                                                            <span class="text-sm text-gray-600" title=variation.name.clone()>{var_name}</span>
                                                            <span class="text-xs text-gray-500 font-mono">{variation.score}"pts"</span>
                                                            <span class="text-xs text-gray-500">v{variation.test_variant}</span>
                                                            {move || {
                                                                if if_show_delete() {
                                                                    view! {
                                                                        <span class="text-xs text-red-600 font-medium">
                                                                            "⚠️ Can delete"
                                                                        </span>
                                                                    }.into_view()
                                                                } else {
                                                                    view! { <span></span> }.into_view()
                                                                }
                                                            }}
                                                        </div>

                                                        <CompactActions
                                                            test=variation.clone()
                                                            is_variation=true
                                                            if_show_delete=if_show_delete
                                                            on_delete_test=on_delete_test.clone()
                                                            on_create_variation=None
                                                        />
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }}
                        </div>
                    }
                }
            />
        </div>
    }
}

#[component]
fn DenseTableView(
    all_tests: Vec<Test>,
    if_show_delete: ReadSignal<bool>,
    on_delete_test: Callback<String>,
    on_create_variation: Callback<Test>,
) -> impl IntoView {
    // Sort tests: base tests first, then variations grouped under them
    let sorted_tests = create_memo(move |_| {
        let tests = all_tests.clone();

        // Separate base tests and variations
        let mut base_tests: Vec<Test> = tests
            .iter()
            .filter(|t| !is_variation_test(t))
            .cloned()
            .collect();

        let variations: Vec<Test> = tests
            .iter()
            .filter(|t| is_variation_test(t))
            .cloned()
            .collect();

        // Sort base tests by name
        base_tests.sort_by(|a, b| a.name.cmp(&b.name));

        // Build final list: base test followed by its variations
        let mut final_tests = Vec::new();

        for base_test in base_tests {
            final_tests.push(base_test.clone());

            // Add variations for this base test
            let mut test_variations: Vec<Test> = variations
                .iter()
                .filter(|v| {
                    let base_name = if v.name.contains(" - ") {
                        v.name.split(" - ").next().unwrap_or(&v.name)
                    } else {
                        &v.name
                    };
                    base_name == base_test.name
                })
                .cloned()
                .collect();

            test_variations.sort_by(|a, b| a.name.cmp(&b.name));
            final_tests.extend(test_variations);
        }

        final_tests
    });

    view! {
        <div class=styles::TABLE_CONTAINER>
            // Table header
            <div class=styles::TABLE_HEADER>
                <div class="grid grid-cols-12 gap-2">
                    <div class=format!("{} col-span-4", styles::TABLE_HEADER_CELL)>"Test Name"</div>
                    <div class=format!("{} col-span-1", styles::TABLE_HEADER_CELL)>"Type"</div>
                    <div class=format!("{} col-span-1", styles::TABLE_HEADER_CELL)>"Score"</div>
                    <div class=format!("{} col-span-1", styles::TABLE_HEADER_CELL)>"Grade"</div>
                    <div class=format!("{} col-span-1", styles::TABLE_HEADER_CELL)>"Variant"</div>
                    <div class=format!("{} col-span-2", styles::TABLE_HEADER_CELL)>"Status"</div>
                    <div class=format!("{} col-span-2", styles::TABLE_HEADER_CELL)>"Actions"</div>
                </div>
            </div>

            // Table body with max height and scroll
            <div class="max-h-[600px] overflow-y-auto">
                <For
                    each=move || sorted_tests()
                    key=|test| test.test_id.clone()
                    children=move |test: Test| {
                        let is_variation = is_variation_test(&test);
                        let (test_type_class, test_type_label) = get_test_type_styling(&test.testarea);

                        // Extract variation info
                        let (display_name, _variation_type) = if is_variation {
                            if let Some(var_part) = test.name.split(" - ").nth(1) {
                                (format!("  ↳ {}", var_part), var_part.to_string())
                            } else {
                                (format!("  ↳ Variation"), "Variation".to_string())
                            }
                        } else {
                            (test.name.clone(), String::new())
                        };

                        let grade_display = test.grade_level.as_ref()
                            .map(|g| format!("{:?}", g))
                            .unwrap_or("—".to_string());

                        view! {
                            <div class=move || {
                                if if_show_delete() {
                                    styles::TABLE_ROW_DELETE
                                } else {
                                    styles::TABLE_ROW
                                }
                            }>
                                <div class="grid grid-cols-12 gap-2 items-center">
                                    // Test Name
                                    <div class=format!("{} col-span-4", styles::TABLE_CELL)>
                                        <div class=move || {
                                            if is_variation {
                                                "text-gray-600 text-sm pl-4"
                                            } else {
                                                "font-medium text-gray-900"
                                            }
                                        }>
                                            <span
                                                title=test.name.clone()
                                                class="truncate block"
                                            >
                                                {display_name}
                                            </span>
                                        </div>
                                    </div>

                                    // Type
                                    <div class=format!("{} col-span-1", styles::TABLE_CELL)>
                                        <span class=format!("inline-flex px-2 py-1 text-xs font-medium rounded-full {}", test_type_class)>
                                            {test_type_label}
                                        </span>
                                    </div>

                                    // Score
                                    <div class=format!("{} col-span-1 font-mono text-center", styles::TABLE_CELL)>
                                        {test.score}
                                    </div>

                                    // Grade
                                    <div class=format!("{} col-span-1 text-center", styles::TABLE_CELL)>
                                        <span class="text-xs">{grade_display}</span>
                                    </div>

                                    // Variant Number
                                    <div class=format!("{} col-span-1 font-mono text-center", styles::TABLE_CELL)>
                                        {if test.test_variant > 0 { test.test_variant.to_string() } else { "—".to_string() }}
                                    </div>

                                    // Status
                                    <div class=format!("{} col-span-2", styles::TABLE_CELL)>
                                        <TestStatusBadges test=test.clone() />
                                    </div>

                                    // Actions
                                    <div class=format!("{} col-span-2", styles::TABLE_CELL)>
                                        <CompactActions
                                            test=test.clone()
                                            is_variation=is_variation
                                            if_show_delete=if_show_delete
                                            on_delete_test=on_delete_test.clone()
                                            on_create_variation=if !is_variation { Some(on_create_variation.clone()) } else { None }
                                        />
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
fn CompactActions(
    test: Test,
    is_variation: bool,
    if_show_delete: ReadSignal<bool>,
    on_delete_test: Callback<String>,
    on_create_variation: Option<Callback<Test>>,
) -> impl IntoView {
    let test_id_edit = test.test_id.clone();
    let test_id_use = test.test_id.clone();
    let test_id_delete = test.test_id.clone();
    let test_for_modal = Rc::new(test.clone());
    let test_for_variation = test.clone();

    //Modal state for selecting a test
    let (show_select_test_modal, set_show_select_test_modal) = create_signal(false);

    view! {
        <div class="flex items-center space-x-1">
            // Edit button
            <button
                class="p-1.5 text-gray-600 hover:text-blue-600 hover:bg-blue-50 rounded transition-colors"
                title="Edit Test"
                on:click=move |_| {
                    let navigate = leptos_router::use_navigate();
                    navigate(&format!("/testbuilder/{}", test_id_edit.clone()), Default::default());
                }
            >
                <span class="text-sm">"✏️"</span>
            </button>

            // Test button (purple button from original)
            <button
                class="p-1.5 text-gray-600 hover:text-purple-600 hover:bg-purple-50 rounded transition-colors"
                title="Test"
                on:click=move |_| set_show_select_test_modal.set(true)
            >
                <span class="text-sm">"🧪"</span>
            </button>

            // Create variation button (only for base tests)
            {if !is_variation && on_create_variation.is_some() {
                let callback = on_create_variation.unwrap();
                let test_clone = test_for_variation.clone();
                view! {
                    <button
                        class="p-1.5 text-gray-600 hover:text-orange-600 hover:bg-orange-50 rounded transition-colors"
                        title="Create Variation"
                        on:click=move |_| leptos::Callable::call(&callback, test_clone.clone())
                    >
                        <span class="text-sm">"📝"</span>
                    </button>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }}

            // Delete button (only in delete mode)
            <Show
                when=move || if_show_delete()
                fallback=|| view! { <span></span> }
            >
                {
                    let delete_id = test_id_delete.clone();
                    let delete_cb = on_delete_test.clone();
                    view! {
                        <button
                            class="p-1.5 text-red-600 hover:bg-red-100 rounded transition-colors"
                            title="Delete Test"
                            on:click=move |_| leptos::Callable::call(&delete_cb, delete_id.clone())
                        >
                            <span class="text-sm">"🗑️"</span>
                        </button>
                    }
                }
            </Show>
        </div>

        // Keep the modal from original TestActionButtons
        <SelectTestModal
            test=test_for_modal
            show_modal=show_select_test_modal
            set_show_modal=set_show_select_test_modal
        />
    }
}

#[component]
fn TestStatusBadges(test: Test) -> impl IntoView {
    let is_variation = is_variation_test(&test);

    view! {
        <div class="flex items-center space-x-1">
            {if is_variation {
                let (var_class, var_label) = get_variation_styling(&test.name);
                view! {
                    <span class=format!("inline-flex px-1.5 py-0.5 text-xs rounded {}", var_class)>
                        {var_label}
                    </span>
                }.into_view()
            } else {
                view! {
                    <span class="inline-flex px-1.5 py-0.5 text-xs bg-gray-100 text-gray-800 rounded">
                        "Base"
                    </span>
                }.into_view()
            }}

            {if !test.comments.is_empty() {
                view! {
                    <span
                        class="inline-flex px-1.5 py-0.5 text-xs bg-yellow-100 text-yellow-800 rounded cursor-help"
                        title=test.comments.clone()
                    >
                        "💬"
                    </span>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }}
        </div>
    }
}

#[component]
fn VariationCard(
    variation: Test,
    if_show_delete: ReadSignal<bool>,
    on_delete_test: Callback<String>,
) -> impl IntoView {
    let variation_name = if variation.name.contains(" - ") {
        let parts: Vec<&str> = variation.name.split(" - ").collect();
        parts
            .get(1)
            .map_or("Variation".to_string(), |v| v.to_string())
    } else {
        "Variation".to_string()
    };

    let (variation_badge_class, variation_label) = get_variation_styling(&variation_name);

    // Clone all the values we need to avoid borrow checker issues
    let variation_test_id_for_delete = variation.test_id.clone();
    let variation_test_id_for_display = variation.test_id.clone();
    let variation_name_display = variation.name.clone();
    let variation_score = variation.score;
    let variation_test_variant = variation.test_variant;
    let variation_comments = variation.comments.clone();
    let variation_for_actions = variation.clone();

    view! {
        <div class="bg-white rounded-lg border border-gray-200 hover:border-gray-300 hover:shadow-sm transition-all duration-200">
            <div class="p-4">
                <div class="flex items-start justify-between mb-3">
                    <span class=format!("inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium border {}", variation_badge_class)>
                        {variation_label}
                    </span>
                    // Enhanced delete button with better visibility
                    {move || {
                        if if_show_delete() {
                            let variation_id = variation_test_id_for_delete.clone();
                            view! {
                                <button
                                    class="px-2 py-1 bg-red-100 hover:bg-red-200 text-red-700 text-xs font-medium rounded-md transition-colors border border-red-300"
                                    on:click=move |_| {
                                        log::info!("Deleting variation: {}", variation_id.clone());
                                        leptos::Callable::call(&on_delete_test, variation_id.clone());
                                    }
                                    title="Delete this variation"
                                >
                                    "🗑️ Delete"
                                </button>
                            }.into_view()
                        } else {
                            view! { <span></span> }.into_view()
                        }
                    }}
                </div>

                <div class="mb-3">
                    <h5 class="font-medium text-gray-900 text-sm mb-1">{variation_name_display}</h5>
                    <p class="text-xs text-gray-600">
                        {format!("{} points • Variant {} • ID: {}", variation_score, variation_test_variant, variation_test_id_for_display)}
                    </p>
                    {if !variation_comments.is_empty() {
                        view! {
                            <p class="text-xs text-gray-500 mt-1 line-clamp-2">{variation_comments}</p>
                        }.into_view()
                    } else {
                        view! { <span></span> }.into_view()
                    }}
                </div>

                <TestActionButtons test=variation_for_actions on_create_variation=None />
            </div>
        </div>
    }
}

#[component]
fn TestActionButtons(test: Test, on_create_variation: Option<Callback<Test>>) -> impl IntoView {
    let test_id_for_edit = test.test_id.clone();
    let test_id_for_use = test.test_id.clone();
    let test_id_for_flash = test.test_id.clone();
    let test_for_variation = test.clone();
    let test_for_modal = Rc::new(test.clone());
    let is_base_test = !is_variation_test(&test);

    //Modal state for selecting a test
    let (show_select_test_modal, set_show_select_test_modal) = create_signal(false);

    view! {
        <div class="flex space-x-2">
            <button
                class="flex-1 px-3 py-2 bg-blue-50 hover:bg-blue-100 text-blue-700 text-xs font-medium rounded-md transition-colors"
                on:click=move |_| {
                    let test_id = test_id_for_edit.clone();
                    let navigate = leptos_router::use_navigate();
                    navigate(&format!("/testbuilder/{}", test_id), Default::default());
                }
            >
                "Edit Test"
            </button>

            <button
                class="flex-1 px-3 py-2 bg-purple-50 hover:bg-purple-100 text-purple-700 text-xs font-medium rounded-md transition-colors"
                on:click=move |_| set_show_select_test_modal.set(true)
            >
                "Test"
            </button>

            {if is_base_test && on_create_variation.is_some() {
                let create_variation_callback = on_create_variation.unwrap();
                view! {
                    <button
                        class="flex-1 px-3 py-2 bg-orange-50 hover:bg-orange-100 text-orange-700 text-xs font-medium rounded-md transition-colors"
                        on:click=move |_| leptos::Callable::call(&create_variation_callback, test_for_variation.clone())
                        title="Create variation"
                    >
                        "New Variation"
                    </button>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }}
        </div>

        // Modal component
        <SelectTestModal
            test=test_for_modal
            show_modal=show_select_test_modal
            set_show_modal=set_show_select_test_modal
        />
    }
}

// =============================================================================
// MAIN COMPONENTS
// =============================================================================

#[component]
pub fn UnifiedTestManager() -> impl IntoView {
    view! {
        <ServerAuthGuard page_path="/test-manager">
            <UnifiedTestManagerContent />
        </ServerAuthGuard>
    }
}

#[component]
pub fn UnifiedTestManagerContent() -> impl IntoView {
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::AdministerTest);

    // State management
    let (if_show_edit, set_if_show_edit) = create_signal(false);
    let (if_show_delete, set_if_show_delete) = create_signal(false);
    let (if_show_toast, set_if_show_toast) = create_signal(false);
    let (toast_message, set_toast_message) = create_signal(ToastMessage::new());
    let (search_term, set_search_term) = create_signal(String::new());
    let (view_mode, set_view_mode) = create_signal(ViewMode::Cards);
    let (test_filter, set_test_filter) = create_signal(TestFilter::All);
    let (expanded_groups, set_expanded_groups) =
        create_signal(std::collections::HashSet::<String>::new());
    let (show_stats, set_show_stats) = create_signal(false);

    //Variation signals
    let (show_create_variation_modal, set_show_create_variation_modal) = create_signal(false);
    let (selected_base_test_for_variation, set_selected_base_test_for_variation) =
        create_signal::<Option<Test>>(None);
    let (variation_type, set_variation_type) = create_signal(String::new());
    let (is_creating_variation, set_is_creating_variation) = create_signal(false);

    let get_tests_info = create_resource(|| (), |_| async move { get_tests().await });

    // Calculate statistics
    let test_stats = create_memo(move |_| {
        let tests_result = get_tests_info.get().unwrap_or(Ok(Vec::new()));
        let tests = tests_result.unwrap_or_default();

        let total_tests = tests.len();
        let math_tests = tests
            .iter()
            .filter(|t| t.testarea == TestType::Math)
            .count();
        let reading_tests = tests
            .iter()
            .filter(|t| t.testarea == TestType::Reading)
            .count();
        let other_tests = tests
            .iter()
            .filter(|t| t.testarea == TestType::Other)
            .count();
        let variations = tests.iter().filter(|t| is_variation_test(t)).count();

        (
            total_tests,
            math_tests,
            reading_tests,
            other_tests,
            variations,
        )
    });

    // Group tests and apply filters
    let test_groups = create_memo(move |_| {
        let tests_result = get_tests_info.get().unwrap_or(Ok(Vec::new()));
        let tests = tests_result.unwrap_or_default();

        let filtered_tests: Vec<Test> = tests
            .into_iter()
            .filter(|test| match test_filter.get() {
                TestFilter::All => true,
                TestFilter::Math => test.testarea == TestType::Math,
                TestFilter::Reading => test.testarea == TestType::Reading,
                TestFilter::Other => test.testarea == TestType::Other,
            })
            .collect();

        let mut groups: std::collections::HashMap<String, TestGroup> =
            std::collections::HashMap::new();

        for test in filtered_tests {
            let base_name = if test.name.contains(" - ") {
                test.name
                    .split(" - ")
                    .next()
                    .unwrap_or(&test.name)
                    .to_string()
            } else {
                test.name.clone()
            };

            if is_variation_test(&test) {
                groups
                    .entry(base_name.clone())
                    .and_modify(|group| group.add_variation(test.clone()))
                    .or_insert_with(|| {
                        let mut group = TestGroup::new(test.clone());
                        group.variations.clear();
                        group.add_variation(test.clone());
                        group
                    });
            } else {
                groups
                    .entry(base_name.clone())
                    .and_modify(|group| {
                        if group.base_test.name.contains(" - ") {
                            group.base_test = test.clone();
                        }
                    })
                    .or_insert_with(|| TestGroup::new(test.clone()));
            }
        }

        let mut sorted_groups: Vec<TestGroup> = groups.into_values().collect();
        sorted_groups.sort_by(|a, b| a.base_test.name.cmp(&b.base_test.name));
        sorted_groups
    });

    // Filter based on search and view mode
    let filtered_display = create_memo(move |_| {
        let groups = test_groups.get();
        let search = search_term.get().to_lowercase();

        let filtered_groups: Vec<TestGroup> = if search.is_empty() {
            groups
        } else {
            groups
                .into_iter()
                .filter(|group| {
                    group.base_test.name.to_lowercase().contains(&search)
                        || group
                            .variations
                            .iter()
                            .any(|v| v.name.to_lowercase().contains(&search))
                })
                .collect()
        };

        // Extract all tests for table/compact views
        let mut all_tests = Vec::new();
        for group in &filtered_groups {
            all_tests.push(group.base_test.clone());
            all_tests.extend(group.variations.clone());
        }

        // Apply test filter to all tests
        let filtered_all_tests: Vec<Test> = all_tests
            .into_iter()
            .filter(|test| match test_filter.get() {
                TestFilter::All => true,
                TestFilter::Math => test.testarea == TestType::Math,
                TestFilter::Reading => test.testarea == TestType::Reading,
                TestFilter::Other => test.testarea == TestType::Other,
            })
            .collect();

        (filtered_all_tests, filtered_groups)
    });

    let create_variation = move |_| {
        if let Some(base_test) = selected_base_test_for_variation.get() {
            set_is_creating_variation(true);

            let base_test_clone = base_test.clone();
            let variation_type_value = variation_type.get();

            spawn_local(async move {
                let variation_name = format!("{} - {}", base_test_clone.name, variation_type_value);
                let variation_comments = format!(
                    "Variation: {} of {}",
                    variation_type_value, base_test_clone.name
                );

                // Determine variant number based on type
                let variant_number =
                    match get_next_variant_number_for_base(&base_test_clone.name).await {
                        Ok(num) => num,
                        Err(e) => {
                            log::error!("Failed to get next variant number: {:?}", e);
                            set_toast_message("Failed to determine variant number".to_string());
                            set_if_show_toast(true);
                            set_is_creating_variation(false);
                            return;
                        }
                    };

                let create_request = CreateNewTestRequest::new(
                    variation_name,
                    base_test_clone.score,
                    base_test_clone.instructions.clone(),
                    variation_comments,
                    base_test_clone.testarea.clone(),
                    base_test_clone.school_year.clone(),
                    base_test_clone.benchmark_categories.clone(),
                    variant_number,
                    base_test_clone.grade_level.clone(),
                    base_test_clone.scope.clone(),
                    base_test_clone.course_id.clone(),
                );

                match add_test(create_request).await {
                    Ok(new_test) => {
                        match variation_type_value.to_lowercase().as_str() {
                            "randomized" => {
                                // For randomized tests, generate questions automatically
                                match duplicate_and_randomize_questions(
                                    base_test_clone.test_id.clone(),
                                    new_test.test_id.clone(),
                                )
                                .await
                                {
                                    Ok(_) => {
                                        get_tests_info.refetch();
                                        set_show_create_variation_modal(false);
                                        set_variation_type(String::new());
                                        set_selected_base_test_for_variation(None);
                                        set_toast_message(ToastMessage::create(
                                            ToastMessageType::NewTestAdded,
                                        ));
                                        set_if_show_toast(true);

                                        // Navigate to test manager to view the completed randomized test
                                        let navigate = leptos_router::use_navigate();
                                        navigate("/test-manager", Default::default());
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "Failed to create randomized questions: {:?}",
                                            e
                                        );
                                        set_toast_message(
                                            "Failed to create randomized questions".to_string(),
                                        );
                                        set_if_show_toast(true);
                                    }
                                }
                            }
                            "distinct" | "practice" => {
                                // For distinct and practice tests, navigate to test builder to add questions
                                get_tests_info.refetch();
                                set_show_create_variation_modal(false);
                                set_variation_type(String::new());
                                set_selected_base_test_for_variation(None);
                                set_toast_message(ToastMessage::create(
                                    ToastMessageType::NewTestAdded,
                                ));
                                set_if_show_toast(true);

                                // Navigate to edit the new variation
                                let navigate = leptos_router::use_navigate();
                                navigate(
                                    &format!("/testbuilder/{}", new_test.test_id),
                                    Default::default(),
                                );
                            }
                            _ => {
                                log::error!("Unknown variation type: {}", variation_type_value);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create variation: {:?}", e);
                        set_toast_message("Failed to create test variation".to_string());
                        set_if_show_toast(true);
                    }
                }

                set_is_creating_variation(false);
            });
        }
    };

    // Event handlers
    let on_click_add = move |_| {
        let navigate = leptos_router::use_navigate();
        navigate("/testbuilder", Default::default());
    };

    let on_click_delete_mode = move |_| {
        set_if_show_delete(!if_show_delete());
        set_if_show_edit(false);
    };

    let on_toggle_view_mode = move |_: ev::MouseEvent| match view_mode.get() {
        ViewMode::Cards => set_view_mode(ViewMode::Table),
        ViewMode::Table => set_view_mode(ViewMode::Compact),
        ViewMode::Compact => set_view_mode(ViewMode::Cards),
    };

    let on_delete_test = Callback::new(move |test_id: String| {
        spawn_local(async move {
            let delete_test_request = DeleteTestRequest::new(test_id);
            match delete_test(delete_test_request).await {
                Ok(_) => {
                    get_tests_info.refetch();
                    set_toast_message(ToastMessage::create(ToastMessageType::TestDeleted));
                    set_if_show_toast(true);
                }
                Err(e) => {
                    log::error!("Error deleting test: {:?}", e);
                    set_if_show_toast(true);
                }
            }
        });
    });

    let toggle_group_expansion = move |base_name: String| {
        set_expanded_groups.update(|expanded| {
            if expanded.contains(&base_name) {
                expanded.remove(&base_name);
            } else {
                expanded.insert(base_name);
            }
        });
    };

    let on_create_variation = Callback::new(move |test: Test| {
        set_selected_base_test_for_variation(Some(test));
        set_show_create_variation_modal(true);
    });

    view! {
        <div class="min-h-screen bg-[#F9F9F8]">
            <Header />
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />
            <div class="ml-0 sm:ml-10 md:ml-30 transition-all duration-300 ease-in-out">
                <div class="max-w-7xl mx-auto px-3 sm:px-6 lg:px-8 py-6 sm:py-12">
                    <Toast
                        toast_message
                        if_appear=if_show_toast
                        set_if_appear=set_if_show_toast
                    />

                    // Page header
                    <div class="pb-4 sm:pb-6 border-b border-gray-200 mb-6 sm:mb-8">
                        <div class="flex flex-col space-y-4">
                            <div>
                                <h1 class="text-3xl sm:text-4xl font-bold text-[#2E3A59]">Test Manager</h1>
                                <p class="mt-2 text-sm sm:text-base text-gray-600">
                                    "Unified test management with variation support and advanced filtering"
                                </p>
                            </div>

                            <StatsPanel
                                test_stats=test_stats
                                show_stats=show_stats
                                set_show_stats=set_show_stats
                            />

                            <FilterTabs
                                test_filter=test_filter
                                set_test_filter=set_test_filter
                            />

                            <div class="flex flex-col sm:flex-row sm:items-center gap-3">
                                // Search bar with fixed width - maintains its size
                                <div class="w-full sm:w-80 flex-shrink-0">
                                    <SearchBar
                                        search_term=search_term
                                        set_search_term=set_search_term
                                    />
                                </div>

                                // Action buttons container - takes remaining space
                                <div class="flex-1 min-w-0">
                                    <ActionButtons
                                        if_show_edit=if_show_edit
                                        if_show_delete=if_show_delete
                                        view_mode=view_mode
                                        set_view_mode=set_view_mode
                                        on_click_add=on_click_add
                                        on_click_delete_mode=on_click_delete_mode
                                    />
                                </div>
                            </div>
                        </div>
                    </div>

                    // Tests display
                    <Suspense fallback=move || {
                        view! {
                            <div class="flex justify-center items-center py-12">
                                <div class="animate-pulse flex space-x-4">
                                    <div class="rounded-full bg-gray-200 h-12 w-12"></div>
                                    <div class="flex-1 space-y-4 py-1">
                                        <div class="h-4 bg-gray-200 rounded w-3/4"></div>
                                        <div class="space-y-2">
                                            <div class="h-4 bg-gray-200 rounded"></div>
                                            <div class="h-4 bg-gray-200 rounded w-5/6"></div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    }>
                        {move || {
                            let (all_tests, grouped_tests) = filtered_display.get();

                            match view_mode.get() {
                                ViewMode::Table => {
                                    if all_tests.is_empty() {
                                        view! {
                                            <EmptyState
                                                test_filter=test_filter
                                                on_click_add=on_click_add
                                            />
                                        }.into_view()
                                    } else {
                                        view! {
                                            <DenseTableView
                                                all_tests=all_tests
                                                if_show_delete=if_show_delete
                                                on_delete_test=on_delete_test
                                                on_create_variation=on_create_variation.clone()
                                            />
                                        }.into_view()
                                    }
                                }
                                ViewMode::Compact => {
                                    if all_tests.is_empty() {
                                        view! {
                                            <EmptyState
                                                test_filter=test_filter
                                                on_click_add=on_click_add
                                            />
                                        }.into_view()
                                    } else {
                                        view! {
                                            <CompactListView
                                                all_tests=all_tests
                                                if_show_delete=if_show_delete
                                                on_delete_test=on_delete_test
                                                on_create_variation=on_create_variation.clone()
                                            />
                                        }.into_view()
                                    }
                                }
                                ViewMode::Cards => {
                                    // Your existing grouped card view
                                    if grouped_tests.is_empty() {
                                        view! {
                                            <EmptyState
                                                test_filter=test_filter
                                                on_click_add=on_click_add
                                            />
                                        }.into_view()
                                    } else {
                                        view! {
                                            <div class="space-y-6">
                                                <For
                                                    each=move || grouped_tests.clone()
                                                    key=|group| group.base_test.test_id.clone()
                                                    children=move |group: TestGroup| {
                                                        view! {
                                                            <TestGroupCard
                                                                group=group
                                                                expanded_groups=expanded_groups
                                                                if_show_delete=if_show_delete
                                                                on_delete_test=on_delete_test
                                                                on_create_variation=on_create_variation.clone()
                                                                toggle_group_expansion=toggle_group_expansion
                                                            />
                                                        }
                                                    }
                                                />
                                            </div>
                                        }.into_view()
                                    }
                                }
                            }
                        }}
                    </Suspense>
                </div>
            </div>

            {move || {
                if show_create_variation_modal() {
                    view! {
                        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                            <div class="bg-white rounded-lg shadow-xl p-6 max-w-md w-full mx-4">
                                <h3 class="text-xl font-semibold text-gray-800 mb-4">Create Test Variation</h3>
                                {selected_base_test_for_variation.get().map(|test| view! {
                                    <p class="text-sm text-gray-600 mb-4">
                                        "Creating variation for: " <strong>{test.name.clone()}</strong>
                                    </p>
                                })}

                                <div class="mb-4">
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        "Variation Type"
                                    </label>
                                    <div class="space-y-3">
                                        // Randomized Variation
                                        <div class="border rounded-lg p-3 hover:bg-gray-50 cursor-pointer"
                                             on:click=move |_| set_variation_type("Randomized".to_string())>
                                            <div class="flex items-start space-x-3">
                                                <input
                                                    type="radio"
                                                    name="variation_type"
                                                    value="Randomized"
                                                    class="mt-1"
                                                    prop:checked=move || variation_type() == "Randomized"
                                                    on:change=move |_| set_variation_type("Randomized".to_string())
                                                />
                                                <div class="flex-1">
                                                    <h4 class="font-medium text-gray-900">Randomized Version</h4>
                                                    <p class="text-sm text-gray-600 mt-1">
                                                        "Same questions with shuffled order and randomized answer choices. Questions are automatically generated."
                                                    </p>
                                                    <span class="inline-block mt-2 px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded">
                                                        "Auto-generated"
                                                    </span>
                                                </div>
                                            </div>
                                        </div>

                                        // Distinct Variation
                                        <div class="border rounded-lg p-3 hover:bg-gray-50 cursor-pointer"
                                             on:click=move |_| set_variation_type("Distinct".to_string())>
                                            <div class="flex items-start space-x-3">
                                                <input
                                                    type="radio"
                                                    name="variation_type"
                                                    value="Distinct"
                                                    class="mt-1"
                                                    prop:checked=move || variation_type() == "Distinct"
                                                    on:change=move |_| set_variation_type("Distinct".to_string())
                                                />
                                                <div class="flex-1">
                                                    <h4 class="font-medium text-gray-900">Distinct Version</h4>
                                                    <p class="text-sm text-gray-600 mt-1">
                                                        "Entirely different questions covering the same topics. You'll add new questions manually."
                                                    </p>
                                                    <span class="inline-block mt-2 px-2 py-1 bg-green-100 text-green-800 text-xs rounded">
                                                        "Manual entry required"
                                                    </span>
                                                </div>
                                            </div>
                                        </div>

                                        // Practice Variation
                                        <div class="border rounded-lg p-3 hover:bg-gray-50 cursor-pointer"
                                             on:click=move |_| set_variation_type("Practice".to_string())>
                                            <div class="flex items-start space-x-3">
                                                <input
                                                    type="radio"
                                                    name="variation_type"
                                                    value="Practice"
                                                    class="mt-1"
                                                    prop:checked=move || variation_type() == "Practice"
                                                    on:change=move |_| set_variation_type("Practice".to_string())
                                                />
                                                <div class="flex-1">
                                                    <h4 class="font-medium text-gray-900">Practice Version</h4>
                                                    <p class="text-sm text-gray-600 mt-1">
                                                        "Practice test for student preparation with new questions. You'll add practice questions manually."
                                                    </p>
                                                    <span class="inline-block mt-2 px-2 py-1 bg-purple-100 text-purple-800 text-xs rounded">
                                                        "Manual entry required"
                                                    </span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div class="flex justify-end space-x-3">
                                    <button
                                        class="px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300"
                                        on:click=move |_| {
                                            set_show_create_variation_modal(false);
                                            set_variation_type(String::new());
                                            set_selected_base_test_for_variation(None);
                                        }
                                        prop:disabled=is_creating_variation
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400"
                                        on:click=create_variation
                                        prop:disabled=move || is_creating_variation() || variation_type().is_empty()
                                    >
                                        {move || if is_creating_variation() {
                                            "Creating..."
                                        } else {
                                            "Create & Edit"
                                        }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }
            }}
        </div>
    }
}
