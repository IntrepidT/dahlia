use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::components::test_item::TestItem;
use crate::app::models::assessment::{
    Assessment, CreateNewAssessmentRequest, DeleteAssessmentRequest, RangeCategory, ScopeEnum,
    SubjectEnum, UpdateAssessmentRequest,
};
use crate::app::models::student::GradeEnum;
use crate::app::models::test::Test;
use crate::app::server_functions::assessments::{
    add_assessment, delete_assessment, get_assessment, get_assessments, update_assessment,
};
use crate::app::server_functions::courses::get_courses;
use crate::app::server_functions::tests::get_tests;
use leptos::*;
use strum::IntoEnumIterator;
use uuid::Uuid;

#[component]
pub fn AssessmentPage() -> impl IntoView {
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::Assessments);
    // Resource to load all assessments
    let assessments_resource = create_resource(|| (), |_| async move { get_assessments().await });

    // Resource to load all tests
    let tests_resource = create_resource(|| (), |_| async move { get_tests().await });
    let courses_resource = create_resource(|| (), |_| async move { get_courses().await });

    // State for assessment form
    let (show_form, set_show_form) = create_signal(false);
    let (editing, set_editing) = create_signal(false);
    let (selected_assessment_id, set_selected_assessment_id) = create_signal::<Option<Uuid>>(None);

    // Form input signals
    let (name, set_name) = create_signal(String::new());
    let (frequency, set_frequency) = create_signal::<Option<i32>>(None);
    let (grade, set_grade) = create_signal::<Option<GradeEnum>>(None);
    let (version, set_version) = create_signal(1);
    let (selected_tests, set_selected_tests) = create_signal::<Vec<Uuid>>(vec![]);
    let (subject, set_subject) = create_signal(SubjectEnum::Other);
    let (risk_benchmarks, set_risk_benchmarks) = create_signal::<Option<Vec<RangeCategory>>>(None);
    let (national_benchmarks, set_national_benchmarks) =
        create_signal::<Option<Vec<RangeCategory>>>(None);

    // Benchmark editing
    let (risk_benchmark_min, set_risk_benchmark_min) = create_signal(0);
    let (risk_benchmark_max, set_risk_benchmark_max) = create_signal(0);
    let (risk_benchmark_label, set_risk_benchmark_label) = create_signal(String::new());

    let (natl_benchmark_min, set_natl_benchmark_min) = create_signal(0);
    let (natl_benchmark_max, set_natl_benchmark_max) = create_signal(0);
    let (natl_benchmark_label, set_natl_benchmark_label) = create_signal(String::new());
    let (scope, set_scope) = create_signal::<Option<ScopeEnum>>(None);
    let (course_id, set_course_id) = create_signal::<Option<i32>>(None);

    // Action to handle form submission
    let submit_form = create_action(move |_: &()| {
        let name_val = name.get();
        let frequency_val = frequency.get();
        let grade_val = grade.get();
        let version_val = version.get();
        let tests_val = selected_tests.get();
        let subject_val = subject.get();
        let risk_val = risk_benchmarks.get();
        let natl_val = national_benchmarks.get();
        let scope = scope.get();
        if scope != Some(ScopeEnum::Course) {
            set_course_id(None); // Reset course_id if scope is not Course
        }
        let course_id = course_id.get();

        // Calculate composite score from selected tests
        let composite = if tests_val.is_empty() {
            None
        } else {
            let tests_resource_value = tests_resource.get();
            tests_resource_value
                .and_then(|result| {
                    result.ok().map(|tests| {
                        // Filter tests by selected IDs and sum their scores
                        let sum: i32 = tests
                            .iter()
                            .filter(|test| {
                                tests_val.contains(
                                    &Uuid::parse_str(&test.test_id).expect("String -> UUID failed"),
                                )
                            })
                            .map(|test| test.score)
                            .sum();

                        Some(sum)
                    })
                })
                .flatten()
        };

        let editing_val = editing.get();
        let selected_id = selected_assessment_id.get();

        async move {
            if editing_val && selected_id.is_some() {
                // Update existing assessment
                let request = UpdateAssessmentRequest::new(
                    name_val,
                    frequency_val,
                    grade_val,
                    version_val,
                    selected_id.unwrap(),
                    tests_val,
                    composite,
                    risk_val,
                    natl_val,
                    subject_val,
                    scope,
                    course_id,
                );
                update_assessment(request).await
            } else {
                // Create new assessment
                let request = CreateNewAssessmentRequest::new(
                    name_val,
                    frequency_val,
                    grade_val,
                    version_val,
                    tests_val,
                    composite,
                    risk_val,
                    natl_val,
                    subject_val,
                    scope,
                    course_id,
                );
                add_assessment(request).await
            }
        }
    });

    // Action to delete assessment
    let delete_action = create_action(|id: &Uuid| {
        let id = *id;
        async move {
            let request = DeleteAssessmentRequest::new(1, id); // Using 1 as default version
            delete_assessment(request).await
        }
    });

    // Reset form function
    let reset_form = move || {
        set_name(String::new());
        set_frequency(None);
        set_grade(None);
        set_version(1);
        set_selected_tests(vec![]);
        set_subject(SubjectEnum::Other);
        set_risk_benchmarks(None);
        set_national_benchmarks(None);
        set_scope(None);
        set_course_id(None);
        set_editing(false);
        set_selected_assessment_id(None);
    };

    // Function to load assessment for editing
    let edit_assessment = move |assessment: Assessment| {
        set_name(assessment.name);
        set_frequency(assessment.frequency);
        set_grade(assessment.grade);
        set_version(assessment.version);
        set_selected_tests(assessment.tests);
        set_subject(assessment.subject);
        set_risk_benchmarks(assessment.risk_benchmarks);
        set_national_benchmarks(assessment.national_benchmarks);
        set_scope(assessment.scope);
        set_course_id(assessment.course_id);
        set_editing(true);
        set_selected_assessment_id(Some(assessment.id));
        set_show_form(true);
    };

    // Add benchmark functions
    let add_risk_benchmark = move |_| {
        let min = risk_benchmark_min.get();
        let max = risk_benchmark_max.get();
        let label = risk_benchmark_label.get();

        if !label.is_empty() {
            let new_benchmark = RangeCategory::new(min, max, label);
            let mut current = risk_benchmarks.get().unwrap_or_default();
            current.push(new_benchmark);
            set_risk_benchmarks(Some(current));

            // Reset inputs
            set_risk_benchmark_min(0);
            set_risk_benchmark_max(0);
            set_risk_benchmark_label(String::new());
        }
    };

    let add_natl_benchmark = move |_| {
        let min = natl_benchmark_min.get();
        let max = natl_benchmark_max.get();
        let label = natl_benchmark_label.get();

        if !label.is_empty() {
            let new_benchmark = RangeCategory::new(min, max, label);
            let mut current = national_benchmarks.get().unwrap_or_default();
            current.push(new_benchmark);
            set_national_benchmarks(Some(current));

            // Reset inputs
            set_natl_benchmark_min(0);
            set_natl_benchmark_max(0);
            set_natl_benchmark_label(String::new());
        }
    };

    // Effect to refresh assessments after submit
    create_effect(move |_| {
        if let Some(Ok(_)) = submit_form.value().get() {
            reset_form();
            set_show_form(false);
            assessments_resource.refetch();
        }
    });

    // Effect to refresh assessments after delete
    create_effect(move |_| {
        if let Some(Ok(_)) = delete_action.value().get() {
            assessments_resource.refetch();
        }
    });

    view! {
        <div class="min-h-screen bg-[#F9F9F8] text-[#2E3A59] font-sans">
            <Header />
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />
            <div class="max-w-6xl mx-auto px-4 py-8">
                <div class="flex justify-between">
                    <h1 class="text-3xl font-medium mb-8 text-[#2E3A59]">"Assessments"</h1>

                    <div class="mb-8">
                        <button
                            class="bg-[#2E3A59] text-white px-4 py-2 rounded shadow-md hover:opacity-90 transition-opacity text-sm font-medium"
                            on:click=move |_| {
                                reset_form();
                                set_show_form(true);
                            }
                        >
                            "Add New Assessment"
                        </button>
                    </div>
                </div>

                // Assessment List
                <div class="bg-white rounded-lg shadow-sm mb-8 overflow-hidden">
                    <div class="border-b border-[#DADADA] px-6 py-4">
                        <h2 class="text-xl font-medium text-[#2E3A59]">"All Assessments"</h2>
                    </div>
                    <div class="p-6">
                        {move || assessments_resource.get().map(|assessments_result| {
                            match assessments_result {
                                Ok(assessments) => {
                                    view! {
                                        <div class="grid grid-cols-1 gap-4">
                                            {assessments.into_iter().map(|assessment| {
                                                let assessment_clone = assessment.clone();
                                                let assessment_id = assessment.id;
                                                let (expanded, set_expanded) = create_signal(false);

                                                view! {
                                                    <div class="bg-white rounded-lg shadow-sm border border-gray-100 hover:shadow-md transition-shadow overflow-hidden">
                                                        <button
                                                            class="w-full text-left p-4 focus:outline-none"
                                                            on:click=move |_| set_expanded.update(|val| *val = !*val)
                                                        >
                                                            <div class="flex justify-between items-center">
                                                                <div>
                                                                    <h3 class="font-medium">{assessment.name}</h3>
                                                                    <p class="text-sm text-gray-500">"("{assessment.subject.to_string()}")"</p>
                                                                </div>
                                                                <div class="text-sm text-gray-500">
                                                                    {assessment.grade.map(|g| format!("{:?}", g)).unwrap_or_else(|| "Any".to_string())}
                                                                </div>
                                                            </div>
                                                        </button>

                                                        // Expandable details section
                                                        <div
                                                            class="border-t border-gray-100 overflow-hidden transition-all duration-300 ease-in-out"
                                                            style:max-height={move || if expanded.get() { "500px" } else { "0" }}
                                                            style:opacity={move || if expanded.get() { "1" } else { "0" }}
                                                        >
                                                            <div class="p-4">
                                                                <div class="flex flex-col md:flex-row gap-4">
                                                                    <div class="flex-grow">
                                                                        <h4 class="text-sm font-medium mb-2">Tests</h4>
                                                                        <div class="max-h-64 overflow-y-auto pr-2 mb-3">
                                                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                                                                                {move || {
                                                                                    // Get all tests
                                                                                    let all_tests = tests_resource.get().map(|r| r.ok()).flatten().unwrap_or_default();

                                                                                    // Get the IDs of tests assigned to this specific assessment
                                                                                    let assessment_test_ids = &assessment.tests;

                                                                                    // First show assessment tests in their original order
                                                                                    let mut ordered_tests: Vec<_> = all_tests.iter()
                                                                                        .filter(|test| {
                                                                                            let test_id = Uuid::parse_str(&test.test_id).expect("Did not convert uuid to string");
                                                                                            assessment_test_ids.contains(&test_id)
                                                                                        })
                                                                                        .collect();

                                                                                    // Sort ordered_tests according to the order in assessment_test_ids
                                                                                    ordered_tests.sort_by(|a, b| {
                                                                                        let a_id = Uuid::parse_str(&a.test_id).expect("Did not convert uuid to string");
                                                                                        let b_id = Uuid::parse_str(&b.test_id).expect("Did not convert uuid to string");
                                                                                        let a_pos = assessment_test_ids.iter().position(|id| *id == a_id).unwrap_or(usize::MAX);
                                                                                        let b_pos = assessment_test_ids.iter().position(|id| *id == b_id).unwrap_or(usize::MAX);
                                                                                        a_pos.cmp(&b_pos)
                                                                                    });

                                                                                    ordered_tests.into_iter().enumerate().map(|(index, test)| {
                                                                                        let test_id = test.test_id.clone();
                                                                                        let test_clone = test.clone();
                                                                                        let test_name = test.name.clone();
                                                                                        view! {
                                                                                            <div class="mb-1">
                                                                                                <TestItem
                                                                                                    test=test_clone.clone()
                                                                                                    test_id=test_id
                                                                                                    test_name=test_name
                                                                                                />
                                                                                            </div>
                                                                                        }
                                                                                    }).collect_view()
                                                                                }}
                                                                            </div>
                                                                        </div>
                                                                    </div>

                                                                    <div class="md:w-72 space-y-4">
                                                                        {move || {
                                                                            if let Some(benchmarks) = &assessment.risk_benchmarks {
                                                                                view! {
                                                                                    <div class="space-y-1">
                                                                                        <h4 class="text-sm font-medium">Risk Benchmarks</h4>
                                                                                        <div class="max-h-28 overflow-y-auto">
                                                                                            {benchmarks.iter().map(|b| {
                                                                                                view! {
                                                                                                    <div class="text-xs flex justify-between">
                                                                                                        <span>{&b.label}</span>
                                                                                                        <span>{b.min} - {b.max}</span>
                                                                                                    </div>
                                                                                                }
                                                                                            }).collect_view()}
                                                                                        </div>
                                                                                    </div>
                                                                                }
                                                                            } else {
                                                                                view! { <div></div> }
                                                                            }
                                                                        }}

                                                                        {move || {
                                                                            if let Some(benchmarks) = &assessment.national_benchmarks {
                                                                                view! {
                                                                                    <div class="space-y-1">
                                                                                        <h4 class="text-sm font-medium">National Benchmarks</h4>
                                                                                        <div class="max-h-28 overflow-y-auto">
                                                                                            {benchmarks.iter().map(|b| {
                                                                                                view! {
                                                                                                    <div class="text-xs flex justify-between">
                                                                                                        <span>{&b.label}</span>
                                                                                                        <span>{b.min} - {b.max}</span>
                                                                                                    </div>
                                                                                                }
                                                                                            }).collect_view()}
                                                                                        </div>
                                                                                    </div>
                                                                                }
                                                                            } else {
                                                                                view! { <div></div> }
                                                                            }
                                                                        }}
                                                                    </div>
                                                                </div>

                                                                <div class="flex justify-end space-x-2 pt-3 mt-2 border-t border-gray-100">
                                                                    <button
                                                                        class="text-xs px-3 py-1 bg-gray-100 rounded-full text-[#2E3A59] hover:bg-gray-200 transition-colors"
                                                                        on:click=move |ev| {
                                                                            ev.stop_propagation();
                                                                            edit_assessment(assessment_clone.clone());
                                                                        }
                                                                    >
                                                                        "Edit"
                                                                    </button>
                                                                    <button
                                                                        class="text-xs px-3 py-1 bg-red-50 rounded-full text-red-600 hover:bg-red-100 transition-colors"
                                                                        on:click=move |ev| {
                                                                            ev.stop_propagation();
                                                                            delete_action.dispatch(assessment_id);
                                                                        }
                                                                    >
                                                                        "Delete"
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }
                                },
                                Err(e) => view! {
                                    <div class="p-4 bg-red-50 text-red-700 rounded border border-red-200">
                                        "Error loading assessments: " {e.to_string()}
                                    </div>
                                }
                            }
                        })}
                    </div>
                </div>

                // Assessment Form
                <div
                    class="bg-white rounded-lg shadow-sm mb-8 overflow-hidden"
                    style:display={move || if show_form.get() { "block" } else { "none" }}
                >
                    <div class="border-b border-[#DADADA] px-6 py-4">
                        <h2 class="text-xl font-medium text-[#2E3A59]">
                            {move || if editing.get() { "Edit Assessment" } else { "Create New Assessment" }}
                        </h2>
                    </div>
                    <div class="p-6">
                        <form on:submit=move |ev| {
                            ev.prevent_default();
                            submit_form.dispatch(());
                        }>
                            <div class="space-y-6">
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                    <div>
                                        <label for="name" class="block text-sm font-medium mb-1">"Name"</label>
                                        <input
                                            type="text"
                                            id="name"
                                            class="w-full px-3 py-2 border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                            prop:value={move || name.get()}
                                            on:input=move |ev| set_name(event_target_value(&ev))
                                            required
                                        />
                                    </div>

                                    <div>
                                        <label for="subject" class="block text-sm font-medium mb-1">"Subject"</label>
                                        <select
                                            required
                                            id="subject"
                                            class="w-full px-3 py-2 border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59] bg-white"
                                            prop:value={move || subject.get().to_string()}
                                            on:change=move |ev| {
                                                let value = event_target_value(&ev);
                                                match value.parse::<SubjectEnum>() {
                                                    Ok(subject_enum) => set_subject(subject_enum),
                                                    Err(_) => ()
                                                }
                                            }
                                        >
                                            <option value="">"Please select a value"</option>
                                            {SubjectEnum::iter().map(|option| view! {
                                                <option value=format!("{}", option)>
                                                    {format!("{}", option)}
                                                </option>
                                            }).collect::<Vec<_>>()}
                                        </select>
                                    </div>

                                    <div>
                                        <label for="grade" class="block text-sm font-medium mb-1">"Grade"</label>
                                        <select
                                            id="grade"
                                            class="w-full px-3 py-2 border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59] bg-white"
                                            prop:value={move || grade.get().map(|g| g.to_string()).unwrap_or_else(|| "".to_string())}
                                            on:change=move |ev| {
                                                let value = event_target_value(&ev);
                                                if value.is_empty() {
                                                    set_grade(None);
                                                } else {
                                                    match value.parse::<GradeEnum>() {
                                                        Ok(grade_enum) => set_grade(Some(grade_enum)),
                                                        Err(_) => set_grade(None)
                                                    }
                                                }
                                            }
                                        >
                                            <option value="">"Please select a value"</option>
                                            {GradeEnum::iter().map(|grade| view! {
                                                <option value=format!("{}", grade)>
                                                    {format!("{}", grade)}
                                                </option>
                                            }).collect::<Vec<_>>()}
                                        </select>
                                    </div>

                                    <div class="grid grid-cols-2 gap-4">
                                        <div>
                                            <label for="frequency" class="block text-sm font-medium mb-1">"Frequency (per year)"</label>
                                            <input
                                                type="number"
                                                id="frequency"
                                                class="w-full px-3 py-2 border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                prop:value={move || frequency.get().unwrap_or(0)}
                                                on:input=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    if value.is_empty() {
                                                        set_frequency(None);
                                                    } else if let Ok(f) = value.parse::<i32>() {
                                                        set_frequency(Some(f));
                                                    }
                                                }
                                            />
                                        </div>

                                        <div>
                                            <label for="version" class="block text-sm font-medium mb-1">"Version"</label>
                                            <input
                                                type="number"
                                                id="version"
                                                min="0"
                                                class="w-full px-3 py-2 border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                prop:value={move || version.get()}
                                                on:input=move |ev| {
                                                    if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                        set_version(v);
                                                    }
                                                }
                                                min="1"
                                                required
                                            />
                                        </div>

                                        <div>
                                            <label for="scope" class="block text-sm font-medium mb-1">"Scope"</label>
                                            <select
                                                id="scope"
                                                class="w-full px-3 py-2 border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59] bg-white"
                                                prop:value={move || scope.get().map(|s| s.to_string()).unwrap_or_else(|| "None".to_string())}
                                                on:change=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    match value.parse::<ScopeEnum>() {
                                                        Ok(scope_enum) => set_scope(Some(scope_enum)),
                                                        Err(_) => set_scope(None)
                                                    }
                                                }
                                            >
                                                <option value="None">"None"</option>
                                                {ScopeEnum::iter().map(|option| view! {
                                                    <option value=format!("{}", option)>
                                                        {format!("{}", option)}
                                                    </option>
                                                }).collect::<Vec<_>>()}
                                            </select>
                                           <Show when=move || matches!(scope(), Some(ScopeEnum::Course))>
                                               <div class="mt-2">
                                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                                        "Course"
                                                    </label>
                                                    <Suspense fallback=move || view! {
                                                        <div class="w-full px-4 py-3 rounded-md border border-gray-300 bg-gray-100 text-gray-500">
                                                            "Loading courses..."
                                                        </div>
                                                    }>
                                                        <select
                                                            class="w-full px-4 py-3 rounded-md border border-gray-300 shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                                            prop:value=move || course_id().map(|id| id.to_string()).unwrap_or_default()
                                                            on:change=move |event| {
                                                                let value = event_target_value(&event);
                                                                if value.is_empty() {
                                                                    set_course_id(None);
                                                                } else if let Ok(id) = value.parse::<i32>() {
                                                                    set_course_id(Some(id));
                                                                }
                                                            }
                                                        >
                                                            <option value="">"Select a Course"</option>
                                                            {move || {
                                                                courses_resource.get()
                                                                    .map(|result| {
                                                                        match result {
                                                                            Ok(courses) => {
                                                                                courses.into_iter().map(|course| {
                                                                                    view! {
                                                                                        <option value=course.id.to_string()>
                                                                                            {course.name.clone()}
                                                                                        </option>
                                                                                    }
                                                                                }).collect::<Vec<_>>()
                                                                            },
                                                                            Err(_) => vec![]
                                                                        }
                                                                    })
                                                                    .unwrap_or_default()
                                                            }}
                                                        </select>
                                                    </Suspense>
                                                </div>
                                            </Show>
                                        </div>
                                    </div>
                                </div>

                                // Tests selection
                                <div>
                                    <label class="block text-sm font-medium mb-3">"Tests"</label>
                                    {move || tests_resource.get().map(|tests_result| {
                                        match tests_result {
                                            Ok(tests) => {
                                                view! {
                                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
                                                        {tests.into_iter().map(|test| {
                                                            let test_id = Uuid::parse_str(&test.test_id).expect("Did not convert uuid to string");
                                                            let test_name = test.name.clone();
                                                            view! {
                                                                <div class="flex items-center space-x-2 p-2 rounded hover:bg-gray-50">
                                                                    <input
                                                                        type="checkbox"
                                                                        id={format!("test-{}", test_id)}
                                                                        class="h-4 w-4 text-[#2E3A59] rounded border-[#DADADA] focus:ring-[#2E3A59]"
                                                                        prop:checked={move || selected_tests.get().contains(&test_id)}
                                                                        on:change=move |ev| {
                                                                            let checked = event_target_checked(&ev);
                                                                            let mut current = selected_tests.get();
                                                                            if checked && !current.contains(&test_id) {
                                                                                current.push(test_id);
                                                                                set_selected_tests(current);
                                                                            } else if !checked {
                                                                                current.retain(|&id| id != test_id);
                                                                                set_selected_tests(current);
                                                                            }
                                                                        }
                                                                    />
                                                                    <label for={format!("test-{}", test_id)} class="text-sm">{test_name}</label>
                                                                </div>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }
                                            },
                                            Err(e) => view! {
                                                <div class="p-4 bg-red-50 text-red-700 rounded border border-red-200">
                                                    "Error loading tests: " {e.to_string()}
                                                </div>
                                            }
                                        }
                                    })}
                                </div>

                                // Risk Benchmarks
                                <div class="border-t border-[#DADADA] pt-6">
                                    <h3 class="text-lg font-medium mb-4">"Risk Benchmarks"</h3>

                                    <div class="space-y-2 mb-4">
                                        {move || {
                                            let benchmarks = risk_benchmarks.get().unwrap_or_default();
                                            benchmarks.into_iter().map(|benchmark| {
                                                let benchmark_label_clone= benchmark.label.clone();
                                                view! {
                                                    <div class="flex items-center justify-between bg-gray-50 p-3 rounded">
                                                        <div class="flex items-center">
                                                            <span class="font-medium text-sm">{benchmark.label}</span>
                                                            <span class="mx-1 text-sm">": "</span>
                                                            <span class="text-sm">{benchmark.min}</span>
                                                            <span class="mx-1 text-sm">" to "</span>
                                                            <span class="text-sm">{benchmark.max}</span>
                                                        </div>
                                                        <button
                                                            type="button"
                                                            class="text-sm text-red-600 hover:opacity-80 transition-opacity"
                                                            on:click=move |_| {
                                                                let mut current = risk_benchmarks.get().unwrap_or_default();
                                                                current.retain(|b| b.label != benchmark_label_clone);
                                                                set_risk_benchmarks(Some(current));
                                                            }
                                                        >
                                                            "Remove"
                                                        </button>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }}
                                    </div>

                                    <div class="flex flex-wrap gap-2 items-end">
                                        <div>
                                            <label class="block text-xs mb-1">"Min"</label>
                                            <input
                                                type="number"
                                                class="w-20 px-2 py-1 text-sm border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                placeholder="Min"
                                                min="0"
                                                prop:value={move || risk_benchmark_min.get()}
                                                on:input=move |ev| {
                                                    if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                        set_risk_benchmark_min(v);
                                                    }
                                                }
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-xs mb-1">"Max"</label>
                                            <input
                                                type="number"
                                                class="w-20 px-2 py-1 text-sm border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                placeholder="Max"
                                                min="0"
                                                prop:value={move || risk_benchmark_max.get()}
                                                on:input=move |ev| {
                                                    if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                        set_risk_benchmark_max(v);
                                                    }
                                                }
                                            />
                                        </div>
                                        <div class="flex-grow">
                                            <label class="block text-xs mb-1">"Label"</label>
                                            <input
                                                type="text"
                                                class="w-full px-2 py-1 text-sm border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                placeholder="Label"
                                                prop:value={move || risk_benchmark_label.get()}
                                                on:input=move |ev| set_risk_benchmark_label(event_target_value(&ev))
                                            />
                                        </div>
                                        <button
                                            type="button"
                                            class="px-3 py-1 bg-gray-100 text-[#2E3A59] text-sm rounded border border-[#DADADA] hover:bg-gray-200 transition-colors"
                                            on:click=add_risk_benchmark
                                        >
                                            "Add"
                                        </button>
                                    </div>
                                </div>

                                // National Benchmarks
                                <div class="border-t border-[#DADADA] pt-6">
                                    <h3 class="text-lg font-medium mb-4">"National Benchmarks"</h3>

                                    <div class="space-y-2 mb-4">
                                        {move || {
                                            let benchmarks = national_benchmarks.get().unwrap_or_default();
                                            benchmarks.into_iter().map(|benchmark| {
                                                let label_clone = benchmark.label.clone();
                                                view! {
                                                    <div class="flex items-center justify-between bg-gray-50 p-3 rounded">
                                                        <div class="flex items-center">
                                                            <span class="font-medium text-sm">{benchmark.label.clone()}</span>
                                                            <span class="mx-1 text-sm">": "</span>
                                                            <span class="text-sm">{benchmark.min}</span>
                                                            <span class="mx-1 text-sm">" to "</span>
                                                            <span class="text-sm">{benchmark.max}</span>
                                                        </div>
                                                        <button
                                                            type="button"
                                                            class="text-sm text-red-600 hover:opacity-80 transition-opacity"
                                                            on:click=move |_| {
                                                                let mut current = national_benchmarks.get().unwrap_or_default();
                                                                current.retain(|b| b.label != label_clone);
                                                                set_national_benchmarks(Some(current));
                                                            }
                                                        >
                                                            "Remove"
                                                        </button>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }}
                                    </div>

                                    <div class="flex flex-wrap gap-2 items-end">
                                        <div>
                                            <label class="block text-xs mb-1">"Min"</label>
                                            <input
                                                type="number"
                                                class="w-20 px-2 py-1 text-sm border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                placeholder="Min"
                                                min="0"
                                                prop:value={move || natl_benchmark_min.get()}
                                                on:input=move |ev| {
                                                    if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                        set_natl_benchmark_min(v);
                                                    }
                                                }
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-xs mb-1">"Max"</label>
                                            <input
                                                type="number"
                                                min="0"
                                                class="w-20 px-2 py-1 text-sm border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                placeholder="Max"
                                                prop:value={move || natl_benchmark_max.get()}
                                                on:input=move |ev| {
                                                    if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                        set_natl_benchmark_max(v);
                                                    }
                                                }
                                            />
                                        </div>
                                        <div class="flex-grow">
                                            <label class="block text-xs mb-1">"Label"</label>
                                            <input
                                                type="text"
                                                class="w-full px-2 py-1 text-sm border border-[#DADADA] rounded focus:outline-none focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                placeholder="Label"
                                                prop:value={move || natl_benchmark_label.get()}
                                                on:input=move |ev| set_natl_benchmark_label(event_target_value(&ev))
                                            />
                                        </div>
                                        <button
                                            type="button"
                                            class="px-3 py-1 bg-gray-100 text-[#2E3A59] text-sm rounded border border-[#DADADA] hover:bg-gray-200 transition-colors"
                                            on:click=add_natl_benchmark
                                        >
                                            "Add"
                                        </button>
                                    </div>
                                </div>

                                <div class="flex justify-end space-x-3 pt-4 border-t border-[#DADADA]">
                                    <button
                                        type="button"
                                        class="px-4 py-2 bg-gray-100 text-[#2E3A59] rounded border border-[#DADADA] hover:bg-gray-200 transition-colors text-sm font-medium"
                                        on:click=move |_| {
                                            set_show_form(false);
                                            reset_form();
                                        }
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        type="submit"
                                        class="px-4 py-2 bg-[#2E3A59] text-white rounded shadow-sm hover:opacity-90 transition-opacity text-sm font-medium"
                                    >
                                        {move || if editing.get() { "Update Assessment" } else { "Create Assessment" }}
                                    </button>
                                </div>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </div>
    }
}
