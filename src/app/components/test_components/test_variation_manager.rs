use crate::app::components::auth::server_auth_components::ServerAuthGuard;
use crate::app::components::header::Header;
use crate::app::models::assessment::ScopeEnum;
use crate::app::models::student::GradeEnum;
use crate::app::models::test::{CreateNewTestRequest, Test, TestType};
use crate::app::server_functions::questions::duplicate_and_randomize_questions;
use crate::app::server_functions::tests::{add_test, get_tests};
use leptos::prelude::*;
use leptos::*;
use leptos_router::*;
use std::str::FromStr;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy, PartialEq)]
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
            VariationType::Randomized => {
                "Same questions with shuffled order and randomized answer choices"
            }
            VariationType::Distinct => "Entirely different questions covering the same topics",
            VariationType::Practice => {
                "Practice version for student preparation with new questions"
            }
        }
    }

    pub fn detailed_description(&self) -> &'static str {
        match self {
            VariationType::Randomized => "Creates a test variation with the same questions but in randomized order with shuffled answer choices. Questions are automatically generated from the base test.",
            VariationType::Distinct => "Creates a blank test variation where you can add entirely new questions for a different version covering the same material.",
            VariationType::Practice => "Creates a blank practice version where you can add new questions for student practice and preparation.",
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
        self.randomized_variations.len()
            + self.distinct_variations.len()
            + self.practice_variations.len()
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

// Utility functions
fn is_variation_test(test: &Test) -> bool {
    test.name.contains(" - ")
        && (VariationType::from_test_name(&test.name).is_some()
            || VariationType::from_comments(&test.comments).is_some())
}

fn get_base_test_name(test_name: &str) -> String {
    if test_name.contains(" - ") {
        test_name
            .split(" - ")
            .next()
            .unwrap_or(test_name)
            .to_string()
    } else {
        test_name.to_string()
    }
}

async fn get_next_variant_number(base_test_name: &str) -> Result<i32, leptos::ServerFnError> {
    let all_tests = get_tests().await?;

    // Find all tests with the same base name (including the base test itself)
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

    // Find the highest variant number among related tests
    let max_variant = related_tests
        .iter()
        .map(|test| test.test_variant)
        .max()
        .unwrap_or(0);

    Ok(max_variant + 1)
}

fn group_tests_by_base(tests: Vec<Test>) -> Vec<TestVariationInfo> {
    let mut groups: std::collections::HashMap<String, TestVariationInfo> =
        std::collections::HashMap::new();

    for test in tests {
        let base_name = get_base_test_name(&test.name);

        if is_variation_test(&test) {
            groups
                .entry(base_name.clone())
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
            groups
                .entry(base_name.clone())
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
    groups
        .into_values()
        .filter(|group| !group.base_test.test_id.is_empty())
        .collect()
}

#[component]
pub fn TestVariationManager() -> impl IntoView {
    view! {
        <ServerAuthGuard page_path="/test-variations">
            <TestVariationManagerContent />
        </ServerAuthGuard>
    }
}

#[component]
pub fn TestVariationManagerContent() -> impl IntoView {
    let (search_term, set_search_term) = create_signal(String::new());
    let (selected_base_test, set_selected_base_test) = create_signal::<Option<Test>>(None);
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (selected_variation_type, set_selected_variation_type) =
        create_signal::<Option<VariationType>>(None);
    let (is_creating, set_is_creating) = create_signal(false);
    let (show_info_panel, set_show_info_panel) = create_signal(false);

    // Load all tests
    let tests_resource = create_resource(
        || (),
        |_| async move {
            match get_tests().await {
                Ok(tests) => tests,
                Err(e) => {
                    log::error!("Failed to load tests: {:?}", e);
                    Vec::new()
                }
            }
        },
    );

    // Group tests into variation families
    let test_groups = create_memo(move |_| {
        let tests = tests_resource.get().unwrap_or_default();
        group_tests_by_base(tests)
    });

    // Filter groups based on search
    let filtered_groups = create_memo(move |_| {
        let groups = test_groups.get();
        let search = search_term.get().to_lowercase();

        if search.is_empty() {
            groups
        } else {
            groups
                .into_iter()
                .filter(|group| {
                    group.base_test.name.to_lowercase().contains(&search)
                        || group
                            .get_all_variations()
                            .iter()
                            .any(|v| v.name.to_lowercase().contains(&search))
                })
                .collect()
        }
    });

    let create_variation = move |_| {
        if let Some(base_test) = selected_base_test.get() {
            if let Some(var_type) = selected_variation_type.get() {
                set_is_creating(true);

                let base_test_clone = base_test.clone();
                let var_type_clone = var_type.clone();

                spawn_local(async move {
                    let variation_name = format!(
                        "{} - {}",
                        base_test_clone.name,
                        var_type_clone.display_name()
                    );
                    let variation_comments = format!(
                        "Variation: {} of {}",
                        var_type_clone.display_name(),
                        base_test_clone.name
                    );

                    // Get the next sequential variant number
                    let variant_number = match get_next_variant_number(&base_test_clone.name).await
                    {
                        Ok(num) => num,
                        Err(e) => {
                            log::error!("Failed to get next variant number: {:?}", e);
                            // Handle error appropriately - you might want to set an error state
                            set_is_creating(false);
                            return;
                        }
                    };

                    let create_request = CreateNewTestRequest::new(
                        variation_name,
                        base_test_clone.score,
                        variation_comments,
                        base_test_clone.testarea.clone(),
                        base_test_clone.school_year.clone(),
                        base_test_clone.benchmark_categories.clone(),
                        variant_number, // Use the correctly named variable
                        base_test_clone.grade_level.clone(),
                        base_test_clone.scope.clone(),
                        base_test_clone.course_id.clone(),
                    );

                    match add_test(create_request).await {
                        Ok(new_test) => {
                            match var_type_clone {
                                VariationType::Randomized => {
                                    // Generate randomized questions automatically
                                    match duplicate_and_randomize_questions(
                                        base_test_clone.test_id.clone(),
                                        new_test.test_id.clone(),
                                    )
                                    .await
                                    {
                                        Ok(_) => {
                                            log::info!("Successfully created randomized variation");
                                            tests_resource.refetch();
                                            set_show_create_modal(false);
                                            set_selected_variation_type(None);
                                            set_selected_base_test(None);
                                        }
                                        Err(e) => {
                                            log::error!(
                                                "Failed to create randomized questions: {:?}",
                                                e
                                            );
                                        }
                                    }
                                }
                                VariationType::Distinct | VariationType::Practice => {
                                    // Navigate to test builder for manual question entry
                                    tests_resource.refetch();
                                    set_show_create_modal(false);
                                    set_selected_variation_type(None);
                                    set_selected_base_test(None);

                                    let navigate = leptos_router::use_navigate();
                                    navigate(
                                        &format!("/testbuilder/{}", new_test.test_id),
                                        Default::default(),
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to create variation: {:?}", e);
                        }
                    }

                    set_is_creating(false);
                });
            }
        }
    };

    view! {
        <Header />
        <main class="w-full max-w-7xl mx-auto px-6 py-12">
            // Header Section
            <div class="flex flex-col mb-8">
                <h1 class="text-3xl font-semibold text-gray-800">Test Variation Manager</h1>
                <p class="mt-2 text-gray-600">Create and manage different versions of your tests</p>
                <div class="h-0.5 w-full bg-gray-300 mt-3"></div>
            </div>

            // Information Panel
            {move || {
                if show_info_panel() {
                    view! {
                        <div class="mb-6 bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg p-6 border border-blue-200">
                            <div class="flex items-start justify-between mb-4">
                                <h3 class="text-lg font-medium text-blue-900">Available Variation Types</h3>
                                <button
                                    class="text-blue-600 hover:text-blue-800 text-sm"
                                    on:click=move |_| set_show_info_panel(false)
                                >
                                    "Hide"
                                </button>
                            </div>
                            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                <VariationTypeInfo var_type=VariationType::Randomized />
                                <VariationTypeInfo var_type=VariationType::Distinct />
                                <VariationTypeInfo var_type=VariationType::Practice />
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="mb-6">
                            <button
                                class="text-blue-600 hover:text-blue-800 text-sm font-medium"
                                on:click=move |_| set_show_info_panel(true)
                            >
                                "Show Variation Types Info"
                            </button>
                        </div>
                    }.into_view()
                }
            }}

            // Search and Controls
            <div class="mb-6 flex flex-col sm:flex-row gap-4">
                <div class="flex-1">
                    <input
                        type="text"
                        placeholder="Search tests and variations..."
                        class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                        prop:value=search_term
                        on:input=move |ev| set_search_term(event_target_value(&ev))
                    />
                </div>
                <button
                    class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    on:click=move |_| {
                        let navigate = leptos_router::use_navigate();
                        navigate("/testbuilder", Default::default());
                    }
                >
                    "Create New Base Test"
                </button>
            </div>

            // Test Groups Display
            <Suspense fallback=move || view! {
                <div class="animate-pulse space-y-4">
                    {(0..3).map(|_| view! {
                        <div class="bg-gray-200 h-32 rounded-lg"></div>
                    }).collect::<Vec<_>>()}
                </div>
            }>
                <div class="space-y-6">
                    <For
                        each=move || filtered_groups.get()
                        key=|group| group.base_test.test_id.clone()
                        children=move |group: TestVariationInfo| {
                            view! {
                                <TestVariationCard
                                    group=group
                                    on_create_variation=move |test: Test| {
                                        set_selected_base_test(Some(test));
                                        set_show_create_modal(true);
                                    }
                                />
                            }
                        }
                    />
                </div>
            </Suspense>

            // Create Variation Modal
            {move || {
                if show_create_modal() {
                    view! {
                        <CreateVariationModal
                            base_test=selected_base_test
                            selected_type=selected_variation_type
                            set_selected_type=set_selected_variation_type
                            is_creating=is_creating
                            on_create=create_variation
                            on_cancel=move |_| {
                                set_show_create_modal(false);
                                set_selected_variation_type(None);
                                set_selected_base_test(None);
                            }
                        />
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </main>
    }
}

#[component]
fn VariationTypeInfo(var_type: VariationType) -> impl IntoView {
    let icon_class = match var_type {
        VariationType::Randomized => "text-blue-600",
        VariationType::Distinct => "text-green-600",
        VariationType::Practice => "text-purple-600",
    };

    view! {
        <div class="bg-white p-4 rounded-lg border border-gray-200">
            <div class="flex items-start space-x-3">
                <div class=format!("flex-shrink-0 w-8 h-8 rounded-full bg-gray-100 flex items-center justify-center {}", icon_class)>
                    {match var_type {
                        VariationType::Randomized => "R",
                        VariationType::Distinct => "D",
                        VariationType::Practice => "P",
                    }}
                </div>
                <div class="flex-1">
                    <h4 class="font-medium text-gray-900">{var_type.display_name()}</h4>
                    <p class="text-sm text-gray-600 mt-1">{var_type.description()}</p>
                    <span class=format!("inline-block mt-2 px-2 py-1 text-xs rounded {}", var_type.badge_class())>
                        {if var_type.requires_manual_questions() { "Manual" } else { "Auto-generated" }}
                    </span>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TestVariationCard(
    group: TestVariationInfo,
    on_create_variation: impl Fn(Test) + 'static + Clone,
) -> impl IntoView {
    let base_test = group.base_test.clone();
    let total_variations = group.total_variations();

    // Clone values we need for the closures
    let base_test_for_create = base_test.clone();
    let base_test_for_edit = base_test.clone();

    view! {
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
            // Base test header
            <div class="bg-gray-50 px-6 py-4 border-b border-gray-200">
                <div class="flex items-center justify-between">
                    <div>
                        <h3 class="text-lg font-medium text-gray-900">{base_test.name.clone()}</h3>
                        <p class="text-sm text-gray-600">
                            {format!("{:?} - {} points - Grade: {:?} - {} variation(s)",
                                base_test.testarea,
                                base_test.score,
                                base_test.grade_level.as_ref().map(|g| format!("{:?}", g)).unwrap_or("Not specified".to_string()),
                                total_variations
                            )}
                        </p>
                    </div>
                    <div class="flex space-x-2">
                        <button
                            class="px-3 py-2 bg-green-600 text-white text-sm rounded-md hover:bg-green-700"
                            on:click=move |_| {
                                let test_clone = base_test_for_create.clone();
                                on_create_variation(test_clone);
                            }
                        >
                            "Create Variation"
                        </button>
                        <button
                            class="px-3 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700"
                            on:click=move |_| {
                                let test_id = base_test_for_edit.test_id.clone();
                                let navigate = leptos_router::use_navigate();
                                navigate(&format!("/testbuilder/{}", test_id), Default::default());
                            }
                        >
                            "Edit Base"
                        </button>
                    </div>
                </div>
            </div>

            // Variations display
            <div class="px-6 py-4">
                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <VariationTypeSection
                        title="Randomized"
                        variations=group.randomized_variations
                        var_type=VariationType::Randomized
                    />
                    <VariationTypeSection
                        title="Distinct"
                        variations=group.distinct_variations
                        var_type=VariationType::Distinct
                    />
                    <VariationTypeSection
                        title="Practice"
                        variations=group.practice_variations
                        var_type=VariationType::Practice
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
fn VariationTypeSection(
    title: &'static str,
    variations: Vec<Test>,
    var_type: VariationType,
) -> impl IntoView {
    view! {
        <div class=format!("border rounded-lg p-4 {}", var_type.card_class())>
            <h4 class="font-medium text-gray-900 mb-3">{title} " (" {variations.len()} ")"</h4>
            {if variations.is_empty() {
                view! {
                    <p class="text-sm text-gray-500 italic">No {title.to_lowercase()} variations yet</p>
                }.into_view()
            } else {
                view! {
                    <div class="space-y-3">
                        <For
                            each=move || variations.clone()
                            key=|variation| variation.test_id.clone()
                            children=move |variation: Test| {
                                view! {
                                    <VariationCard variation=variation var_type=var_type.clone() />
                                }
                            }
                        />
                    </div>
                }.into_view()
            }}
        </div>
    }
}

#[component]
fn VariationCard(variation: Test, var_type: VariationType) -> impl IntoView {
    let variation_clone = variation.clone();

    view! {
        <div class="bg-white rounded-md p-3 border border-gray-200">
            <div class="flex items-center justify-between mb-2">
                <h5 class="font-medium text-gray-900 text-sm">
                    {variation.name.split(" - ").nth(1).unwrap_or("Variation").to_string()}
                </h5>
                <span class=format!("text-xs px-2 py-1 rounded {}", var_type.badge_class())>
                    v{variation.test_variant}
                </span>
            </div>
            <p class="text-xs text-gray-600 mb-3">
                {format!("{} points", variation.score)}
            </p>
            <div class="flex space-x-2">
                <button
                    class="flex-1 px-2 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700"
                    on:click=move |_| {
                        let test_id = variation_clone.test_id.clone();
                        let navigate = leptos_router::use_navigate();
                        navigate(&format!("/testbuilder/{}", test_id), Default::default());
                    }
                >
                    "Edit"
                </button>
                <button
                    class="flex-1 px-2 py-1 bg-green-600 text-white text-xs rounded hover:bg-green-700"
                    on:click=move |_| {
                        let test_id = variation.test_id.clone();
                        let navigate = leptos_router::use_navigate();
                        navigate(&format!("/test-session/{}", test_id), Default::default());
                    }
                >
                    "Use"
                </button>
            </div>
        </div>
    }
}

#[component]
fn CreateVariationModal(
    base_test: ReadSignal<Option<Test>>,
    selected_type: ReadSignal<Option<VariationType>>,
    set_selected_type: WriteSignal<Option<VariationType>>,
    is_creating: ReadSignal<bool>,
    on_create: impl Fn(ev::MouseEvent) + 'static,
    on_cancel: impl Fn(ev::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white rounded-lg shadow-xl p-6 max-w-lg w-full mx-4">
                <h3 class="text-xl font-semibold text-gray-800 mb-4">Create Test Variation</h3>

                {base_test.get().map(|test| view! {
                    <p class="text-sm text-gray-600 mb-6">
                        "Creating variation for: " <strong>{test.name.clone()}</strong>
                    </p>
                })}

                <div class="mb-6">
                    <label class="block text-sm font-medium text-gray-700 mb-3">
                        "Choose Variation Type"
                    </label>
                    <div class="space-y-3">
                        <VariationTypeSelector
                            var_type=VariationType::Randomized
                            selected=selected_type
                            set_selected=set_selected_type
                        />
                        <VariationTypeSelector
                            var_type=VariationType::Distinct
                            selected=selected_type
                            set_selected=set_selected_type
                        />
                        <VariationTypeSelector
                            var_type=VariationType::Practice
                            selected=selected_type
                            set_selected=set_selected_type
                        />
                    </div>
                </div>

                <div class="flex justify-end space-x-3">
                    <button
                        class="px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300"
                        on:click=on_cancel
                        prop:disabled=is_creating
                    >
                        "Cancel"
                    </button>
                    <button
                        class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400"
                        on:click=on_create
                        prop:disabled=move || is_creating() || selected_type().is_none()
                    >
                        {move || {
                            if is_creating() {
                                "Creating...".to_string()
                            } else {
                                match selected_type() {
                                    Some(VariationType::Randomized) => "Create & View".to_string(),
                                    Some(VariationType::Distinct) | Some(VariationType::Practice) => "Create & Edit".to_string(),
                                    None => "Select Type".to_string(),
                                }
                            }
                        }}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn VariationTypeSelector(
    var_type: VariationType,
    selected: ReadSignal<Option<VariationType>>,
    set_selected: WriteSignal<Option<VariationType>>,
) -> impl IntoView {
    let is_selected = create_memo(move |_| selected.get() == Some(var_type));

    view! {
        <div
            class=move || {
                let base_class = "border rounded-lg p-4 cursor-pointer transition-all hover:border-gray-400";
                if is_selected() {
                    format!("{} border-blue-500 bg-blue-50 ring-2 ring-blue-500", base_class)
                } else {
                    format!("{} border-gray-300", base_class)
                }
            }
            on:click=move |_| set_selected(Some(var_type))
        >
            <div class="flex items-start space-x-3">
                <input
                    type="radio"
                    class="mt-1"
                    prop:checked=is_selected
                    on:change=move |_| set_selected(Some(var_type))
                />
                <div class="flex-1">
                    <div class="flex items-center space-x-2 mb-2">
                        <h4 class="font-medium text-gray-900">{var_type.display_name()}</h4>
                        <span class=format!("text-xs px-2 py-1 rounded {}", var_type.badge_class())>
                            {if var_type.requires_manual_questions() { "Manual" } else { "Auto" }}
                        </span>
                    </div>
                    <p class="text-sm text-gray-600">{var_type.detailed_description()}</p>
                </div>
            </div>
        </div>
    }
}
