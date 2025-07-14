use crate::app::components::auth::server_auth_components::ServerAuthGuard;
use crate::app::components::header::Header;
use crate::app::models::test::{Test, TestType, CreateNewTestRequest};
use crate::app::models::student::GradeEnum;
use crate::app::models::assessment::ScopeEnum;
use crate::app::server_functions::tests::{get_tests, add_test};
use leptos::prelude::*;
use leptos::*;
use leptos_router::*;
use std::str::FromStr;
use strum::IntoEnumIterator;

#[component]
pub fn TestVariationManager() -> impl IntoView {
    view! {
        <ServerAuthGuard page_path="/test-variations">
            <TestVariationManagerContent />
        </ServerAuthGuard>
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestVariation {
    pub base_test: Test,
    pub variations: Vec<Test>,
}

impl TestVariation {
    pub fn new(base_test: Test) -> Self {
        Self {
            base_test,
            variations: Vec::new(),
        }
    }
    
    pub fn add_variation(&mut self, variation: Test) {
        self.variations.push(variation);
    }
    
    pub fn get_all_tests(&self) -> Vec<&Test> {
        let mut all_tests = vec![&self.base_test];
        all_tests.extend(self.variations.iter());
        all_tests
    }
    
    pub fn get_variation_by_type(&self, variation_type: &str) -> Option<&Test> {
        self.variations.iter().find(|test| {
            test.comments.to_lowercase().contains(&format!("variation: {}", variation_type.to_lowercase()))
        })
    }
}

#[component]
pub fn TestVariationManagerContent() -> impl IntoView {
    let (selected_base_test, set_selected_base_test) = create_signal::<Option<Test>>(None);
    let (show_create_variation_modal, set_show_create_variation_modal) = create_signal(false);
    let (variation_type, set_variation_type) = create_signal(String::new());
    let (search_term, set_search_term) = create_signal(String::new());
    let (is_creating_variation, set_is_creating_variation) = create_signal(false);
    
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
    
    // Group tests by base name to identify variations
    let test_groups = create_memo(move |_| {
        let tests = tests_resource.get().unwrap_or_default();
        let mut groups: std::collections::HashMap<String, TestVariation> = std::collections::HashMap::new();
        
        for test in tests {
            // Extract base name (everything before " - " if it exists)
            let base_name = if test.name.contains(" - ") {
                test.name.split(" - ").next().unwrap_or(&test.name).to_string()
            } else {
                test.name.clone()
            };
            
            // Determine if this is a base test or variation
            let is_variation = test.name.contains(" - ") && 
                (test.name.to_lowercase().contains("remediation") ||
                 test.name.to_lowercase().contains("advanced") ||
                 test.name.to_lowercase().contains("easy") ||
                 test.name.to_lowercase().contains("hard") ||
                 test.comments.to_lowercase().contains("variation:"));
            
            if is_variation {
                groups.entry(base_name.clone())
                    .and_modify(|group| group.add_variation(test.clone()))
                    .or_insert_with(|| {
                        let mut group = TestVariation::new(test.clone());
                        group.variations.clear(); // Clear since we used it as placeholder
                        group.add_variation(test.clone());
                        group
                    });
            } else {
                groups.entry(base_name.clone())
                    .and_modify(|group| {
                        // If we already have this group but it was created from a variation,
                        // replace the base test
                        if group.base_test.name.contains(" - ") {
                            group.base_test = test.clone();
                        }
                    })
                    .or_insert_with(|| TestVariation::new(test.clone()));
            }
        }
        
        groups.into_values().collect::<Vec<_>>()
    });
    
    // Filter groups based on search
    let filtered_groups = create_memo(move |_| {
        let groups = test_groups.get();
        let search = search_term.get().to_lowercase();
        
        if search.is_empty() {
            groups
        } else {
            groups.into_iter()
                .filter(|group| {
                    group.base_test.name.to_lowercase().contains(&search) ||
                    group.variations.iter().any(|v| v.name.to_lowercase().contains(&search))
                })
                .collect()
        }
    });
    
    let create_variation = move |_| {
        if let Some(base_test) = selected_base_test.get() {
            set_is_creating_variation(true);
            
            let base_test_clone = base_test.clone();
            let variation_type_value = variation_type.get();
            
            spawn_local(async move {
                let variation_name = format!("{} - {}", base_test_clone.name, variation_type_value);
                let variation_comments = format!("Variation: {} of {}", variation_type_value, base_test_clone.name);
                
                // Determine variant number based on type
                let variant_number = match variation_type_value.to_lowercase().as_str() {
                    "remediation" | "easy" => base_test_clone.test_variant + 100,
                    "advanced" | "hard" => base_test_clone.test_variant + 200,
                    "practice" => base_test_clone.test_variant + 300,
                    _ => base_test_clone.test_variant + 10,
                };
                
                let create_request = CreateNewTestRequest::new(
                    variation_name,
                    base_test_clone.score, // Start with same max score
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
                        log::info!("Created variation: {}", new_test.test_id);
                        tests_resource.refetch();
                        set_show_create_variation_modal(false);
                        set_variation_type(String::new());
                        set_selected_base_test(None);
                        
                        // Navigate to edit the new variation
                        let navigate = leptos_router::use_navigate();
                        navigate(&format!("/testbuilder/{}", new_test.test_id), Default::default());
                    }
                    Err(e) => {
                        log::error!("Failed to create variation: {:?}", e);
                    }
                }
                
                set_is_creating_variation(false);
            });
        }
    };
    
    view! {
        <Header />
        <main class="w-full max-w-7xl mx-auto px-6 py-12">
            <div class="flex flex-col mb-8">
                <h1 class="text-3xl font-semibold text-gray-800">Test Variation Manager</h1>
                <p class="mt-2 text-gray-600">Create and manage different versions of your tests</p>
                <div class="h-0.5 w-full bg-gray-300 mt-3"></div>
            </div>
            
            // Search and filters
            <div class="mb-6 flex flex-col sm:flex-row gap-4">
                <div class="flex-1">
                    <input
                        type="text"
                        placeholder="Search tests..."
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
            
            // Test groups display
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
                        children=move |group: TestVariation| {
                            let base_test = group.base_test.clone();
                            let variations = group.variations.clone();
                            let base_test_for_modal = base_test.clone();
                            
                            view! {
                                <div class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
                                    // Base test header
                                    <div class="bg-gray-50 px-6 py-4 border-b border-gray-200">
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <h3 class="text-lg font-medium text-gray-900">{base_test.name.clone()}</h3>
                                                <p class="text-sm text-gray-600">
                                                    {format!("{:?} • {} points • Grade: {:?}", 
                                                        base_test.testarea, 
                                                        base_test.score,
                                                        base_test.grade_level.as_ref().map(|g| format!("{:?}", g)).unwrap_or("Not specified".to_string())
                                                    )}
                                                </p>
                                            </div>
                                            <div class="flex space-x-2">
                                                <button
                                                    class="px-3 py-2 bg-green-600 text-white text-sm rounded-md hover:bg-green-700"
                                                    on:click=move |_| {
                                                        set_selected_base_test(Some(base_test_for_modal.clone()));
                                                        set_show_create_variation_modal(true);
                                                    }
                                                >
                                                    "Create Variation"
                                                </button>
                                                <button
                                                    class="px-3 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700"
                                                    on:click=move |_| {
                                                        let test_id = base_test.test_id.clone();
                                                        let navigate = leptos_router::use_navigate();
                                                        navigate(&format!("/testbuilder/{}", test_id), Default::default());
                                                    }
                                                >
                                                    "Edit Base"
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                    
                                    // Variations list
                                    <div class="px-6 py-4">
                                        {if variations.is_empty() {
                                            view! {
                                                <div class="text-center py-8 text-gray-500">
                                                    <p class="text-sm">No variations created yet</p>
                                                    <p class="text-xs mt-1">Click "Create Variation" to add different versions of this test</p>
                                                </div>
                                            }
                                        } else {
                                            view! {
                                                <div>
                                                    <h4 class="text-sm font-medium text-gray-700 mb-3">Variations:</h4>
                                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                                        <For
                                                            each=move || variations.clone()
                                                            key=|variation| variation.test_id.clone()
                                                            children=move |variation: Test| {
                                                                let variation_clone = variation.clone();
                                                                view! {
                                                                    <div class="bg-gray-50 rounded-md p-4 border border-gray-200">
                                                                        <div class="flex items-center justify-between mb-2">
                                                                            <h5 class="font-medium text-gray-900 text-sm">
                                                                                {variation.name.split(" - ").nth(1).unwrap_or("Variation").to_string()}
                                                                            </h5>
                                                                            <span class="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
                                                                                "v{variation.test_variant}"
                                                                            </span>
                                                                        </div>
                                                                        <p class="text-xs text-gray-600 mb-3">
                                                                            {variation.comments.clone()}
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
                                                                                class="flex-1 px-2 py-1 bg-gray-600 text-white text-xs rounded hover:bg-gray-700"
                                                                                on:click=move |_| {
                                                                                    // TODO: Implement preview functionality
                                                                                }
                                                                            >
                                                                                "Preview"
                                                                            </button>
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            }
                                        }}
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </Suspense>
            
            // Create variation modal
            {move || {
                if show_create_variation_modal() {
                    view! {
                        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                            <div class="bg-white rounded-lg shadow-xl p-6 max-w-md w-full mx-4">
                                <h3 class="text-xl font-semibold text-gray-800 mb-4">Create Test Variation</h3>
                                {selected_base_test.get().map(|test| view! {
                                    <p class="text-sm text-gray-600 mb-4">
                                        "Creating variation for: " <strong>{test.name.clone()}</strong>
                                    </p>
                                })}
                                
                                <div class="mb-4">
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        "Variation Type"
                                    </label>
                                    <select
                                        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                        prop:value=variation_type
                                        on:change=move |ev| set_variation_type(event_target_value(&ev))
                                    >
                                        <option value="">"Select variation type"</option>
                                        <option value="Remediation">"Remediation (Easier)"</option>
                                        <option value="Advanced">"Advanced (Harder)"</option>
                                        <option value="Practice">"Practice Version"</option>
                                        <option value="Timed">"Timed Version"</option>
                                        <option value="Randomized">"Randomized Version"</option>
                                    </select>
                                </div>
                                
                                <div class="flex justify-end space-x-3">
                                    <button
                                        class="px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300"
                                        on:click=move |_| {
                                            set_show_create_variation_modal(false);
                                            set_variation_type(String::new());
                                            set_selected_base_test(None);
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
        </main>
    }
}
