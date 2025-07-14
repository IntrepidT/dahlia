use crate::app::components::auth::server_auth_components::ServerAuthGuard;
use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::components::test_item::TestItem;
use crate::app::models::assessment::{
    Assessment, CreateNewAssessmentRequest, DeleteAssessmentRequest, RangeCategory, ScopeEnum,
    SubjectEnum, UpdateAssessmentRequest,
};
use crate::app::models::assessment_sequences::{SequenceBehavior, TestSequenceItem};
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
    view! {
        <ServerAuthGuard page_path="/assessments">
            <AssessmentPageContent />
        </ServerAuthGuard>
    }
}

#[component]
pub fn AssessmentPageContent() -> impl IntoView {
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::Assessments);

    // Resources
    let assessments_resource =
        create_local_resource(|| (), |_| async move { get_assessments().await });
    let tests_resource = create_local_resource(|| (), |_| async move { get_tests().await });
    let courses_resource = create_local_resource(|| (), |_| async move { get_courses().await });

    // Form state
    let (show_form, set_show_form) = create_signal(false);
    let (editing, set_editing) = create_signal(false);
    let (selected_assessment_id, set_selected_assessment_id) = create_signal::<Option<Uuid>>(None);
    let (use_sequences, set_use_sequences) = create_signal(false);

    // Basic form inputs
    let (name, set_name) = create_signal(String::new());
    let (frequency, set_frequency) = create_signal::<Option<i32>>(None);
    let (grade, set_grade) = create_signal::<Option<GradeEnum>>(None);
    let (version, set_version) = create_signal(1);
    let (subject, set_subject) = create_signal(SubjectEnum::Other);
    let (scope, set_scope) = create_signal::<Option<ScopeEnum>>(None);
    let (course_id, set_course_id) = create_signal::<Option<i32>>(None);

    // Legacy test selection (for non-sequence mode)
    let (selected_tests, set_selected_tests) = create_signal::<Vec<Uuid>>(vec![]);

    // New sequence-based test management
    let (test_sequence, set_test_sequence) = create_signal::<Vec<TestSequenceItem>>(vec![]);
    let (sequence_counter, set_sequence_counter) = create_signal(1);

    // Benchmark state
    let (risk_benchmarks, set_risk_benchmarks) = create_signal::<Option<Vec<RangeCategory>>>(None);
    let (national_benchmarks, set_national_benchmarks) =
        create_signal::<Option<Vec<RangeCategory>>>(None);
    let (risk_benchmark_min, set_risk_benchmark_min) = create_signal(0);
    let (risk_benchmark_max, set_risk_benchmark_max) = create_signal(0);
    let (risk_benchmark_label, set_risk_benchmark_label) = create_signal(String::new());
    let (natl_benchmark_min, set_natl_benchmark_min) = create_signal(0);
    let (natl_benchmark_max, set_natl_benchmark_max) = create_signal(0);
    let (natl_benchmark_label, set_natl_benchmark_label) = create_signal(String::new());

    // Sequence form inputs
    let (selected_test_for_sequence, set_selected_test_for_sequence) =
        create_signal::<Option<Uuid>>(None);
    let (sequence_behavior, set_sequence_behavior) = create_signal(SequenceBehavior::Node);
    let (required_score, set_required_score) = create_signal::<Option<i32>>(None);
    let (max_attempts, set_max_attempts) = create_signal(1);
    let (time_limit, set_time_limit) = create_signal::<Option<i32>>(None);

    // Function to add test to sequence
    let add_test_to_sequence = move |_| {
        if let Some(test_id) = selected_test_for_sequence.get() {
            let order = sequence_counter.get();

            let new_item = match sequence_behavior.get() {
                SequenceBehavior::Attainment => TestSequenceItem::new_attainment(
                    test_id,
                    order,
                    required_score.get().unwrap_or(70),
                    None,
                    None,
                ),
                SequenceBehavior::Node => TestSequenceItem::new_node(test_id, order),
                SequenceBehavior::Optional => TestSequenceItem::new_optional(test_id, order),
                SequenceBehavior::Diagnostic => TestSequenceItem::new_diagnostic(test_id, order),
                SequenceBehavior::Remediation => {
                    TestSequenceItem::new_remediation(test_id, order, vec![])
                }
                SequenceBehavior::Branching => {
                    TestSequenceItem::new_branching(test_id, order, vec![])
                }
            };

            let mut current_sequence = test_sequence.get();
            current_sequence.push(new_item);
            current_sequence.sort_by_key(|item| item.sequence_order);
            set_test_sequence(current_sequence);
            set_sequence_counter(order + 1);

            // Reset form
            set_selected_test_for_sequence(None);
            set_sequence_behavior(SequenceBehavior::Node);
            set_required_score(None);
        }
    };

    // Function to remove test from sequence
    let remove_from_sequence = move |test_id: Uuid| {
        let mut current = test_sequence.get();
        current.retain(|item| item.test_id != test_id);
        set_test_sequence(current);
    };

    // Updated submit form action
    let submit_form = create_action(move |_: &()| {
        let name_val = name.get();
        let frequency_val = frequency.get();
        let grade_val = grade.get();
        let version_val = version.get();
        let subject_val = subject.get();
        let risk_val = risk_benchmarks.get();
        let natl_val = national_benchmarks.get();
        let scope_val = scope.get();
        let course_id_val = if scope_val != Some(ScopeEnum::Course) {
            None
        } else {
            course_id.get()
        };
        let editing_val = editing.get();
        let selected_id = selected_assessment_id.get();
        let use_sequences_val = use_sequences.get();

        async move {
            if use_sequences_val {
                // Use sequence-based creation/update
                let sequence = test_sequence.get();
                let composite = if sequence.is_empty() {
                    None
                } else {
                    let tests_resource_value = tests_resource.get();
                    tests_resource_value
                        .and_then(|result| {
                            result.ok().map(|tests| {
                                let sum: i32 = tests
                                    .iter()
                                    .filter(|test| {
                                        sequence.iter().any(|seq_item| {
                                            let test_uuid =
                                                Uuid::parse_str(&test.test_id).unwrap_or_default();
                                            seq_item.test_id == test_uuid
                                        })
                                    })
                                    .map(|test| test.score)
                                    .sum();
                                Some(sum)
                            })
                        })
                        .flatten()
                };

                if editing_val && selected_id.is_some() {
                    let request = UpdateAssessmentRequest::new_with_sequence(
                        name_val,
                        frequency_val,
                        grade_val,
                        version_val,
                        selected_id.unwrap(),
                        composite,
                        risk_val,
                        natl_val,
                        subject_val,
                        scope_val,
                        course_id_val,
                        sequence,
                    );
                    update_assessment(request).await
                } else {
                    let request = CreateNewAssessmentRequest::new_with_sequence(
                        name_val,
                        frequency_val,
                        grade_val,
                        version_val,
                        composite,
                        risk_val,
                        natl_val,
                        subject_val,
                        scope_val,
                        course_id_val,
                        sequence,
                    );
                    add_assessment(request).await
                }
            } else {
                // Use legacy vector-based creation/update
                let tests_val = selected_tests.get();
                let composite = if tests_val.is_empty() {
                    None
                } else {
                    let tests_resource_value = tests_resource.get();
                    tests_resource_value
                        .and_then(|result| {
                            result.ok().map(|tests| {
                                let sum: i32 = tests
                                    .iter()
                                    .filter(|test| {
                                        tests_val.contains(
                                            &Uuid::parse_str(&test.test_id)
                                                .expect("String -> UUID failed"),
                                        )
                                    })
                                    .map(|test| test.score)
                                    .sum();
                                Some(sum)
                            })
                        })
                        .flatten()
                };

                if editing_val && selected_id.is_some() {
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
                        scope_val,
                        course_id_val,
                    );
                    update_assessment(request).await
                } else {
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
                        scope_val,
                        course_id_val,
                    );
                    add_assessment(request).await
                }
            }
        }
    });

    // Delete action (unchanged)
    let delete_action = create_action(|id: &Uuid| {
        let id = *id;
        async move {
            let request = DeleteAssessmentRequest::new(1, id);
            delete_assessment(request).await
        }
    });

    // Enhanced reset form function
    let reset_form = move || {
        set_name(String::new());
        set_frequency(None);
        set_grade(None);
        set_version(1);
        set_selected_tests(vec![]);
        set_test_sequence(vec![]);
        set_sequence_counter(1);
        set_subject(SubjectEnum::Other);
        set_risk_benchmarks(None);
        set_national_benchmarks(None);
        set_scope(None);
        set_course_id(None);
        set_use_sequences(false);
        set_editing(false);
        set_selected_assessment_id(None);
    };

    // Enhanced edit function
    let edit_assessment = move |assessment: Assessment| {
        set_name(assessment.name);
        set_frequency(assessment.frequency);
        set_grade(assessment.grade);
        set_version(assessment.version);
        set_subject(assessment.subject);
        set_risk_benchmarks(assessment.risk_benchmarks);
        set_national_benchmarks(assessment.national_benchmarks);
        set_scope(assessment.scope);
        set_course_id(assessment.course_id);
        set_editing(true);
        set_selected_assessment_id(Some(assessment.id));

        // Check if assessment uses sequences
        if let Some(sequence) = assessment.test_sequence {
            if !sequence.is_empty() {
                set_use_sequences(true);
                set_test_sequence(sequence);
                let max_order = test_sequence
                    .get()
                    .iter()
                    .map(|item| item.sequence_order)
                    .max()
                    .unwrap_or(0);
                set_sequence_counter(max_order + 1);
            } else {
                set_use_sequences(false);
                set_selected_tests(assessment.tests);
            }
        } else {
            set_use_sequences(false);
            set_selected_tests(assessment.tests);
        }

        set_show_form(true);
    };

    // Benchmark functions (unchanged)
    let add_risk_benchmark = move |_| {
        let min = risk_benchmark_min.get();
        let max = risk_benchmark_max.get();
        let label = risk_benchmark_label.get();

        if !label.is_empty() {
            let new_benchmark = RangeCategory::new(min, max, label);
            let mut current = risk_benchmarks.get().unwrap_or_default();
            current.push(new_benchmark);
            set_risk_benchmarks(Some(current));

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

            set_natl_benchmark_min(0);
            set_natl_benchmark_max(0);
            set_natl_benchmark_label(String::new());
        }
    };

    // Effects (unchanged)
    create_effect(move |_| {
        if let Some(Ok(_)) = submit_form.value().get() {
            reset_form();
            set_show_form(false);
            assessments_resource.refetch();
        }
    });

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

                // Assessment List - Enhanced to show sequence info
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
                                                let uses_sequences = assessment.test_sequence.is_some() &&
                                                    !assessment.test_sequence.as_ref().unwrap().is_empty();

                                                view! {
                                                    <div class="bg-white rounded-lg shadow-sm border border-gray-100 hover:shadow-md transition-shadow overflow-hidden">
                                                        <button
                                                            class="w-full text-left p-4 focus:outline-none"
                                                            on:click=move |_| set_expanded.update(|val| *val = !*val)
                                                        >
                                                            <div class="flex justify-between items-center">
                                                                <div>
                                                                    <h3 class="font-medium">{assessment.name}</h3>
                                                                    <div class="flex items-center space-x-2 text-sm text-gray-500">
                                                                        <span>"("{assessment.subject.to_string()}")"</span>
                                                                        {if uses_sequences {
                                                                            view! { <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded text-xs">"Sequenced"</span> }
                                                                        } else {
                                                                            view! { <span class="bg-gray-100 text-gray-600 px-2 py-1 rounded text-xs">"Legacy"</span> }
                                                                        }}
                                                                    </div>
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
                                                                        {if uses_sequences {
                                                                            view! {
                                                                                <div>
                                                                                    <h4 class="text-sm font-medium mb-2">"Test Sequence"</h4>
                                                                                    <div class="max-h-64 overflow-y-auto pr-2 mb-3">
                                                                                        {if let Some(sequence) = &assessment.test_sequence {
                                                                                            let all_tests = tests_resource.get().map(|r| r.ok()).flatten().unwrap_or_default();
                                                                                            sequence.iter().enumerate().map(|(index, seq_item)| {
                                                                                                let test = all_tests.iter().find(|t| {
                                                                                                    Uuid::parse_str(&t.test_id).unwrap_or_default() == seq_item.test_id
                                                                                                });

                                                                                                view! {
                                                                                                    <div class="flex items-center space-x-2 p-2 bg-gray-50 rounded mb-2">
                                                                                                        <span class="bg-[#2E3A59] text-white rounded-full w-6 h-6 flex items-center justify-center text-xs">
                                                                                                            {seq_item.sequence_order}
                                                                                                        </span>
                                                                                                        <div class="flex-grow">
                                                                                                            <div class="text-sm font-medium">
                                                                                                                {test.map(|t| t.name.clone()).unwrap_or_else(|| "Unknown Test".to_string())}
                                                                                                            </div>
                                                                                                            <div class="flex items-center space-x-2 text-xs text-gray-500">
                                                                                                                <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded">
                                                                                                                    {format!("{:?}", seq_item.sequence_behavior)}
                                                                                                                </span>
                                                                                                                {if let Some(score) = seq_item.required_score {
                                                                                                                    view! { <span>"Req: "{score}</span> }
                                                                                                                } else {
                                                                                                                    view! { <span></span> }
                                                                                                                }}
                                                                                                            </div>
                                                                                                        </div>
                                                                                                    </div>
                                                                                                }
                                                                                            }).collect_view()
                                                                                        } else {
                                                                                            view! { <div>"No sequence defined"</div> }.into_view()
                                                                                        }}
                                                                                    </div>
                                                                                </div>
                                                                            }
                                                                        } else {
                                                                            view! {
                                                                                <div>
                                                                                    <h4 class="text-sm font-medium mb-2">"Tests"</h4>
                                                                                    <div class="max-h-64 overflow-y-auto pr-2 mb-3">
                                                                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                                                                                            {move || {
                                                                                                let all_tests = tests_resource.get().map(|r| r.ok()).flatten().unwrap_or_default();
                                                                                                let assessment_test_ids = &assessment.tests;

                                                                                                let mut ordered_tests: Vec<_> = all_tests.iter()
                                                                                                    .filter(|test| {
                                                                                                        let test_id = Uuid::parse_str(&test.test_id).expect("Did not convert uuid to string");
                                                                                                        assessment_test_ids.contains(&test_id)
                                                                                                    })
                                                                                                    .collect();

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
                                                                            }
                                                                        }}
                                                                    </div>

                                                                    // Benchmarks section (unchanged)
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

                // Enhanced Assessment Form
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
                                // Basic info section
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

                                // Test Management Mode Selection
                                <div class="border-t border-[#DADADA] pt-6">
                                    <div class="mb-4">
                                        <h3 class="text-lg font-medium mb-3">"Test Management"</h3>
                                        <div class="flex items-center space-x-4">
                                            <label class="inline-flex items-center">
                                                <input
                                                    type="radio"
                                                    name="test_mode"
                                                    class="form-radio h-4 w-4 text-[#2E3A59]"
                                                    prop:checked={move || !use_sequences.get()}
                                                    on:change=move |_| set_use_sequences(false)
                                                />
                                                <span class="ml-2 text-sm">"Simple Test List"</span>
                                            </label>
                                            <label class="inline-flex items-center">
                                                <input
                                                    type="radio"
                                                    name="test_mode"
                                                    class="form-radio h-4 w-4 text-[#2E3A59]"
                                                    prop:checked={move || use_sequences.get()}
                                                    on:change=move |_| set_use_sequences(true)
                                                />
                                                <span class="ml-2 text-sm">"Advanced Sequencing"</span>
                                            </label>
                                        </div>
                                        <p class="text-xs text-gray-500 mt-1">
                                            "Advanced sequencing allows you to control test flow with requirements and branching logic."
                                        </p>
                                    </div>

                                    // Conditional Test Selection UI
                                    <Show
                                        when=move || !use_sequences.get()
                                        fallback=move || view! {
                                            // Advanced Sequence Builder
                                            <div>
                                                <h4 class="text-md font-medium mb-3">"Test Sequence Builder"</h4>

                                                // Current Sequence Display
                                                <div class="mb-4">
                                                    <div class="bg-gray-50 rounded-lg p-4 min-h-32">
                                                        <h5 class="text-sm font-medium mb-2">"Current Sequence:"</h5>
                                                        {move || {
                                                            let sequence = test_sequence.get();
                                                            if sequence.is_empty() {
                                                                view! {
                                                                    <div class="text-gray-500 text-sm italic">"No tests in sequence. Add tests below."</div>
                                                                }
                                                            } else {
                                                                let all_tests = tests_resource.get().map(|r| r.ok()).flatten().unwrap_or_default();
                                                                view! {
                                                                    <div class="space-y-2">
                                                                        {sequence.into_iter().enumerate().map(|(index, seq_item)| {
                                                                            let test = all_tests.iter().find(|t| {
                                                                                Uuid::parse_str(&t.test_id).unwrap_or_default() == seq_item.test_id
                                                                            });
                                                                            let item_test_id = seq_item.test_id;

                                                                            view! {
                                                                                <div class="flex items-center justify-between bg-white p-3 rounded border">
                                                                                    <div class="flex items-center space-x-3">
                                                                                        <span class="bg-[#2E3A59] text-white rounded-full w-8 h-8 flex items-center justify-center text-sm font-medium">
                                                                                            {seq_item.sequence_order}
                                                                                        </span>
                                                                                        <div>
                                                                                            <div class="font-medium text-sm">
                                                                                                {test.map(|t| t.name.clone()).unwrap_or_else(|| "Unknown Test".to_string())}
                                                                                            </div>
                                                                                            <div class="flex items-center space-x-2 text-xs text-gray-500">
                                                                                                <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded">
                                                                                                    {format!("{:?}", seq_item.sequence_behavior)}
                                                                                                </span>
                                                                                                {if let Some(score) = seq_item.required_score {
                                                                                                    view! { <span class="bg-green-100 text-green-800 px-2 py-1 rounded">"Required: "{score}</span> }
                                                                                                } else {
                                                                                                    view! { <span></span> }
                                                                                                }}
                                                                                                {if let Some(attempts) = seq_item.max_attempts {
                                                                                                    view! { <span class="bg-orange-100 text-orange-800 px-2 py-1 rounded">"Max: "{attempts}" attempts"</span> }
                                                                                                } else {
                                                                                                    view! { <span></span> }
                                                                                                }}
                                                                                            </div>
                                                                                        </div>
                                                                                    </div>
                                                                                    <button
                                                                                        type="button"
                                                                                        class="text-red-600 hover:text-red-800 text-sm"
                                                                                        on:click=move |_| remove_from_sequence(item_test_id)
                                                                                    >
                                                                                        "Remove"
                                                                                    </button>
                                                                                </div>
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                }
                                                            }
                                                        }}
                                                    </div>
                                                </div>

                                                // Add Test to Sequence Form
                                                <div class="border border-gray-200 rounded-lg p-4">
                                                    <h5 class="text-sm font-medium mb-3">"Add Test to Sequence"</h5>
                                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                                                        <div>
                                                            <label class="block text-xs font-medium mb-1">"Test"</label>
                                                            <select
                                                                class="w-full px-2 py-2 text-sm border border-gray-300 rounded focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                                prop:value={move || selected_test_for_sequence.get().map(|id| id.to_string()).unwrap_or_default()}
                                                                on:change=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    if value.is_empty() {
                                                                        set_selected_test_for_sequence(None);
                                                                    } else if let Ok(uuid) = Uuid::parse_str(&value) {
                                                                        set_selected_test_for_sequence(Some(uuid));
                                                                    }
                                                                }
                                                            >
                                                                <option value="">"Select a test"</option>
                                                                {move || {
                                                                    tests_resource.get().map(|tests_result| {
                                                                        match tests_result {
                                                                            Ok(tests) => {
                                                                                let current_sequence = test_sequence.get();
                                                                                let used_test_ids: Vec<Uuid> = current_sequence.iter().map(|item| item.test_id).collect();

                                                                                tests.into_iter()
                                                                                    .filter(|test| {
                                                                                        let test_uuid = Uuid::parse_str(&test.test_id).unwrap_or_default();
                                                                                        !used_test_ids.contains(&test_uuid)
                                                                                    })
                                                                                    .map(|test| {
                                                                                        view! {
                                                                                            <option value=test.test_id.clone()>
                                                                                                {test.name.clone()}
                                                                                            </option>
                                                                                        }
                                                                                    }).collect_view()
                                                                            },
                                                                            Err(_) => view! {}.into_view()
                                                                        }
                                                                    }).unwrap_or_default()
                                                                }}
                                                            </select>
                                                        </div>

                                                        <div>
                                                            <label class="block text-xs font-medium mb-1">"Behavior"</label>
                                                            <select
                                                                class="w-full px-2 py-2 text-sm border border-gray-300 rounded focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                                prop:value={move || format!("{:?}", sequence_behavior.get())}
                                                                on:change=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    match value.as_str() {
                                                                        "Node" => set_sequence_behavior(SequenceBehavior::Node),
                                                                        "Attainment" => set_sequence_behavior(SequenceBehavior::Attainment),
                                                                        "Optional" => set_sequence_behavior(SequenceBehavior::Optional),
                                                                        "Diagnostic" => set_sequence_behavior(SequenceBehavior::Diagnostic),
                                                                        "Remediation" => set_sequence_behavior(SequenceBehavior::Remediation),
                                                                        "Branching" => set_sequence_behavior(SequenceBehavior::Branching),
                                                                        _ => {}
                                                                    }
                                                                }
                                                            >
                                                                {SequenceBehavior::iter().map(|behavior| {
                                                                    view! {
                                                                        <option value=format!("{:?}", behavior)>
                                                                            {format!("{:?}", behavior)}
                                                                        </option>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </select>
                                                        </div>

                                                        <Show when=move || matches!(sequence_behavior.get(), SequenceBehavior::Attainment)>
                                                            <div>
                                                                <label class="block text-xs font-medium mb-1">"Required Score"</label>
                                                                <input
                                                                    type="number"
                                                                    class="w-full px-2 py-2 text-sm border border-gray-300 rounded focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                                    placeholder="70"
                                                                    prop:value={move || required_score.get().unwrap_or(70)}
                                                                    on:input=move |ev| {
                                                                        if let Ok(score) = event_target_value(&ev).parse::<i32>() {
                                                                            set_required_score(Some(score));
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                        </Show>

                                                        <div>
                                                            <label class="block text-xs font-medium mb-1">"Max Attempts"</label>
                                                            <input
                                                                type="number"
                                                                class="w-full px-2 py-2 text-sm border border-gray-300 rounded focus:ring-1 focus:ring-[#2E3A59] focus:border-[#2E3A59]"
                                                                placeholder="1"
                                                                min="1"
                                                                prop:value={move || max_attempts.get()}
                                                                on:input=move |ev| {
                                                                    if let Ok(attempts) = event_target_value(&ev).parse::<i32>() {
                                                                        set_max_attempts(attempts);
                                                                    }
                                                                }
                                                            />
                                                        </div>
                                                    </div>

                                                    <div class="mt-3 flex justify-end">
                                                        <button
                                                            type="button"
                                                            class="bg-[#2E3A59] text-white px-4 py-2 rounded text-sm hover:opacity-90 transition-opacity"
                                                            on:click=add_test_to_sequence
                                                            disabled=move || selected_test_for_sequence.get().is_none()
                                                        >
                                                            "Add to Sequence"
                                                        </button>
                                                    </div>
                                                </div>

                                                <div class="mt-3 text-xs text-gray-600">
                                                    <p><strong>"Node"</strong>": Standard test - students progress automatically"</p>
                                                    <p><strong>"Attainment"</strong>": Students must achieve the required score to progress"</p>
                                                    <p><strong>"Optional"</strong>": Students can skip this test"</p>
                                                    <p><strong>"Diagnostic"</strong>": Assessment only - doesn't block progression"</p>
                                                </div>
                                            </div>
                                        }
                                    >
                                        // Simple Test Selection (Legacy Mode)
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
                                    </Show>
                                </div>

                                // Risk Benchmarks (unchanged)
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

                                // National Benchmarks (unchanged)
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
