use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::components::server_auth_components::ServerAuthGuard;
use crate::app::components::test_item::TestItem;
use crate::app::models::assessment::{
    Assessment, CreateNewAssessmentRequest, DeleteAssessmentRequest, RangeCategory, ScopeEnum,
    SubjectEnum, UpdateAssessmentRequest,
};
use crate::app::models::assessment_sequences::{
    ScoreRange, SequenceBehavior, TestSequenceItem, VariationLevel,
};
use crate::app::models::student::GradeEnum;
use crate::app::models::test::Test;
use crate::app::server_functions::assessments::{
    add_assessment, delete_assessment, get_assessment, get_assessments, update_assessment,
};
use crate::app::server_functions::courses::get_courses;
use crate::app::server_functions::tests::get_tests;
use itertools::Itertools;
use leptos::*;
use strum::IntoEnumIterator;
use uuid::Uuid;

fn is_variation_test(test: &Test) -> bool {
    test.name.contains(" - ")
        && (test.name.to_lowercase().contains("randomized")
            || test.name.to_lowercase().contains("distinct")
            || test.name.to_lowercase().contains("practice")
            || test.comments.to_lowercase().contains("variation:"))
}

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

    // Modal state
    let (show_modal, set_show_modal) = create_signal(false);
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

    // Sequence builder state
    let (dragging_item, set_dragging_item) = create_signal::<Option<usize>>(None);
    let (show_sequence_details, set_show_sequence_details) = create_signal::<Option<usize>>(None);

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
    let add_test_to_sequence = move |_: leptos::ev::MouseEvent| {
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
        // Reorder sequence
        for (index, item) in current.iter_mut().enumerate() {
            item.sequence_order = (index + 1) as i32;
        }
        set_test_sequence(current);
    };

    // Drag and drop functions
    let on_drag_start = move |index: usize| {
        set_dragging_item(Some(index));
    };

    let on_drag_over = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
    };

    let on_drop = move |target_index: usize| {
        if let Some(source_index) = dragging_item.get() {
            if source_index != target_index {
                let mut sequence = test_sequence.get();
                let item = sequence.remove(source_index);
                sequence.insert(target_index, item);

                // Reorder sequence numbers
                for (index, item) in sequence.iter_mut().enumerate() {
                    item.sequence_order = (index + 1) as i32;
                }

                set_test_sequence(sequence);
            }
        }
        set_dragging_item(None);
    };

    // Function to connect tests with branching
    let connect_tests = move |from_index: usize, to_test_id: Uuid, connection_type: &str| {
        let mut sequence = test_sequence.get();
        if let Some(from_item) = sequence.get_mut(from_index) {
            match connection_type {
                "pass" => from_item.next_on_pass = Some(to_test_id),
                "fail" => from_item.next_on_fail = Some(to_test_id),
                _ => {}
            }
        }
        set_test_sequence(sequence);
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

    // Delete action
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
        set_show_sequence_details(None);
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

        set_show_modal(true);
    };

    // Benchmark functions
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

    // Effects
    create_effect(move |_| {
        if let Some(Ok(_)) = submit_form.value().get() {
            reset_form();
            set_show_modal(false);
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
                            set_show_modal(true);
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
                                                            <div class="flex items-center space-x-2">
                                                                <div class="text-sm text-gray-500">
                                                                    {assessment.grade.map(|g| format!("{:?}", g)).unwrap_or_else(|| "Any".to_string())}
                                                                </div>
                                                                <button
                                                                    class="text-xs px-3 py-1 bg-blue-100 rounded-full text-blue-600 hover:bg-blue-200 transition-colors"
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
                                                    </button>

                                                    // Expandable details section with visual flow
                                                    <div
                                                        class="border-t border-gray-100 overflow-hidden transition-all duration-300 ease-in-out"
                                                        style:max-height={move || if expanded.get() { "800px" } else { "0" }}
                                                        style:opacity={move || if expanded.get() { "1" } else { "0" }}
                                                    >
                                                        <div class="p-4">
                                                            {if uses_sequences {
                                                                view! {
                                                                    <div>
                                                                        <h4 class="text-sm font-medium mb-3">"Test Sequence Flow"</h4>
                                                                        <SequenceVisualization
                                                                            sequence={assessment.test_sequence.clone().unwrap_or_default()}
                                                                            tests={tests_resource.get().map(|r| r.ok()).flatten().unwrap_or_default()}
                                                                        />
                                                                    </div>
                                                                }.into_view()
                                                            } else {
                                                                view! {
                                                                    <div>
                                                                        <h4 class="text-sm font-medium mb-2">"Tests"</h4>
                                                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                                                                            {move || {
                                                                                let all_tests = tests_resource.get().map(|r| r.ok()).flatten().unwrap_or_default();
                                                                                assessment.tests.iter().map(|test_id| {
                                                                                    if let Some(test) = all_tests.iter().find(|t| {
                                                                                        Uuid::parse_str(&t.test_id).unwrap_or_default() == *test_id
                                                                                    }) {
                                                                                        view! {
                                                                                            <TestItem
                                                                                                test=test.clone()
                                                                                                test_id=test.test_id.clone()
                                                                                                test_name=test.name.clone()
                                                                                            />
                                                                                        }.into_view()
                                                                                    } else {
                                                                                        view! { <div>"Unknown Test"</div> }.into_view()
                                                                                    }
                                                                                }).collect_view()
                                                                            }}
                                                                        </div>
                                                                    </div>
                                                                }.into_view()
                                                            }}
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
        </div>

        // Modal for Assessment Form
            <Show when=move || show_modal.get()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50 modal-overlay">
                    <div class="bg-white rounded-lg shadow-xl max-w-6xl w-full max-h-[90vh] flex flex-col modal-content">
                        // Fixed sticky header with proper z-index and styling
                        <div class="sticky top-0 bg-white border-b border-gray-200 px-6 py-4 flex justify-between items-center z-10 rounded-t-lg">
                            <h2 class="text-xl font-medium text-[#2E3A59]">
                                {move || if editing.get() { "Edit Assessment" } else { "Create New Assessment" }}
                            </h2>
                            <button
                                class="text-gray-400 hover:text-gray-600 text-2xl leading-none p-1 hover:bg-gray-100 rounded transition-colors"
                                on:click=move |_| {
                                    set_show_modal(false);
                                    reset_form();
                                }
                            >
                                "×"
                            </button>
                        </div>

                        // Scrollable content area with WHITE background
                        <div class="flex-1 overflow-y-auto bg-white">
                            <div class="p-6 bg-white">
                                <form on:submit=move |ev| {
                                    ev.prevent_default();
                                    submit_form.dispatch(());
                                }>
                                    <div class="space-y-6">
                                        // Basic info section with WHITE background and dark text
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 bg-white p-4 rounded-lg">
                                            <div>
                                                <label for="name" class="block text-sm font-medium mb-1 text-gray-700">"Name"</label>
                                                <input
                                                    type="text"
                                                    id="name"
                                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] text-gray-900 bg-white"
                                                    prop:value={move || name.get()}
                                                    on:input=move |ev| set_name(event_target_value(&ev))
                                                    required
                                                />
                                            </div>

                                            <div>
                                                <label for="subject" class="block text-sm font-medium mb-1 text-gray-700">"Subject"</label>
                                                <select
                                                    required
                                                    id="subject"
                                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] bg-white text-gray-900"
                                                    prop:value={move || subject.get().to_string()}
                                                    on:change=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        match value.parse::<SubjectEnum>() {
                                                            Ok(subject_enum) => set_subject(subject_enum),
                                                            Err(_) => ()
                                                        }
                                                    }
                                                >
                                                    <option value="" class="text-gray-900 bg-white">"Please select a value"</option>
                                                    {SubjectEnum::iter().map(|option| view! {
                                                        <option value=format!("{}", option) class="text-gray-900 bg-white">
                                                            {format!("{}", option)}
                                                        </option>
                                                    }).collect::<Vec<_>>()}
                                                </select>
                                            </div>

                                            <div>
                                                <label for="grade" class="block text-sm font-medium mb-1 text-gray-700">"Grade"</label>
                                                <select
                                                    id="grade"
                                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] bg-white text-gray-900"
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
                                                    <option value="" class="text-gray-900 bg-white">"Please select a value"</option>
                                                    {GradeEnum::iter().map(|grade| view! {
                                                        <option value=format!("{}", grade) class="text-gray-900 bg-white">
                                                            {format!("{}", grade)}
                                                        </option>
                                                    }).collect::<Vec<_>>()}
                                                </select>
                                            </div>

                                            <div class="grid grid-cols-2 gap-4">
                                                <div>
                                                    <label for="frequency" class="block text-sm font-medium mb-1 text-gray-700">"Frequency (per year)"</label>
                                                    <input
                                                        type="number"
                                                        id="frequency"
                                                        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] text-gray-900 bg-white"
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
                                                    <label for="version" class="block text-sm font-medium mb-1 text-gray-700">"Version"</label>
                                                    <input
                                                        type="number"
                                                        id="version"
                                                        min="1"
                                                        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] text-gray-900 bg-white"
                                                        prop:value={move || version.get()}
                                                        on:input=move |ev| {
                                                            if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                                set_version(v);
                                                            }
                                                        }
                                                        required
                                                    />
                                                </div>
                                            </div>

                                            <div>
                                                <label for="scope" class="block text-sm font-medium mb-1 text-gray-700">"Scope"</label>
                                                <select
                                                    id="scope"
                                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-[#2E3A59] focus:border-[#2E3A59] bg-white text-gray-900"
                                                    prop:value={move || scope.get().map(|s| s.to_string()).unwrap_or_else(|| "None".to_string())}
                                                    on:change=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        match value.parse::<ScopeEnum>() {
                                                            Ok(scope_enum) => set_scope(Some(scope_enum)),
                                                            Err(_) => set_scope(None)
                                                        }
                                                    }
                                                >
                                                    <option value="None" class="text-gray-900 bg-white">"None"</option>
                                                    {ScopeEnum::iter().map(|option| view! {
                                                        <option value=format!("{}", option) class="text-gray-900 bg-white">
                                                            {format!("{}", option)}
                                                        </option>
                                                    }).collect::<Vec<_>>()}
                                                </select>
                                            </div>
                                        </div>

                                        // Test Management Mode Selection with WHITE background
                                        <div class="border-t border-gray-200 pt-6 bg-white">
                                            <div class="mb-4 bg-white p-4 rounded-lg">
                                                <h3 class="text-lg font-medium mb-3 text-gray-900">"Test Management"</h3>
                                                <div class="flex items-center space-x-4">
                                                    <label class="inline-flex items-center">
                                                        <input
                                                            type="radio"
                                                            name="test_mode"
                                                            class="form-radio h-4 w-4 text-[#2E3A59] focus:ring-[#2E3A59] border-gray-300"
                                                            prop:checked={move || !use_sequences.get()}
                                                            on:change=move |_| set_use_sequences(false)
                                                        />
                                                        <span class="ml-2 text-sm text-gray-700">"Simple Test List"</span>
                                                    </label>
                                                    <label class="inline-flex items-center">
                                                        <input
                                                            type="radio"
                                                            name="test_mode"
                                                            class="form-radio h-4 w-4 text-[#2E3A59] focus:ring-[#2E3A59] border-gray-300"
                                                            prop:checked={move || use_sequences.get()}
                                                            on:change=move |_| set_use_sequences(true)
                                                        />
                                                        <span class="ml-2 text-sm text-gray-700">"Advanced Sequencing"</span>
                                                    </label>
                                                </div>
                                                <p class="text-xs text-gray-500 mt-1">
                                                    "Advanced sequencing allows you to control test flow with requirements and branching logic."
                                                </p>
                                            </div>

                                            // Conditional Test Selection UI - Force WHITE background
                                            <div class="bg-white">
                                                <Show
                                                    when=move || !use_sequences.get()
                                                    fallback=move || view! {
                                                        // Advanced Visual Sequence Builder with WHITE background
                                                        <div class="bg-white p-4 rounded-lg">
                                                            <AdvancedSequenceBuilder
                                                                test_sequence=test_sequence
                                                                set_test_sequence=set_test_sequence
                                                                tests_resource=tests_resource
                                                                sequence_counter=sequence_counter
                                                                set_sequence_counter=set_sequence_counter
                                                                dragging_item=dragging_item
                                                                set_dragging_item=set_dragging_item
                                                                show_sequence_details=show_sequence_details
                                                                set_show_sequence_details=set_show_sequence_details
                                                                on_drag_start=on_drag_start
                                                                on_drag_over=on_drag_over
                                                                on_drop=on_drop
                                                                connect_tests=connect_tests
                                                                remove_from_sequence=remove_from_sequence
                                                            />
                                                        </div>
                                                    }
                                                >
                                                    // Simple Test Selection with WHITE background
                                                    <div class="bg-white p-4 rounded-lg">
                                                        <label class="block text-sm font-medium mb-3 text-gray-700">"Tests"</label>
                                                        {move || tests_resource.get().map(|tests_result| {
                                                            match tests_result {
                                                                Ok(tests) => {
                                                                    view! {
                                                                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2 max-h-64 overflow-y-auto p-3 bg-gray-50 rounded-md border border-gray-200">
                                                                            {tests.into_iter().map(|test| {
                                                                                let test_id = Uuid::parse_str(&test.test_id).expect("Did not convert uuid to string");
                                                                                let test_name = test.name.clone();
                                                                                view! {
                                                                                    <div class="flex items-center space-x-2 p-2 rounded hover:bg-white transition-colors">
                                                                                        <input
                                                                                            type="checkbox"
                                                                                            id={format!("test-{}", test_id)}
                                                                                            class="h-4 w-4 text-[#2E3A59] rounded border-gray-300 focus:ring-[#2E3A59]"
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
                                                                                        <label for={format!("test-{}", test_id)} class="text-sm text-gray-700 cursor-pointer">{test_name}</label>
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
                                        </div>

                                        // Risk Benchmarks with WHITE background
                                        <div class="border-t border-gray-200 pt-6 bg-white p-4 rounded-lg">
                                            <h3 class="text-lg font-medium mb-4 text-gray-900">"Risk Benchmarks"</h3>
                                            <BenchmarkSection
                                                benchmarks=risk_benchmarks
                                                set_benchmarks=set_risk_benchmarks
                                                benchmark_min=risk_benchmark_min
                                                set_benchmark_min=set_risk_benchmark_min
                                                benchmark_max=risk_benchmark_max
                                                set_benchmark_max=set_risk_benchmark_max
                                                benchmark_label=risk_benchmark_label
                                                set_benchmark_label=set_risk_benchmark_label
                                                add_benchmark=add_risk_benchmark
                                            />
                                        </div>

                                        // National Benchmarks with WHITE background
                                        <div class="border-t border-gray-200 pt-6 bg-white p-4 rounded-lg">
                                            <h3 class="text-lg font-medium mb-4 text-gray-900">"National Benchmarks"</h3>
                                            <BenchmarkSection
                                                benchmarks=national_benchmarks
                                                set_benchmarks=set_national_benchmarks
                                                benchmark_min=natl_benchmark_min
                                                set_benchmark_min=set_natl_benchmark_min
                                                benchmark_max=natl_benchmark_max
                                                set_benchmark_max=set_natl_benchmark_max
                                                benchmark_label=natl_benchmark_label
                                                set_benchmark_label=set_natl_benchmark_label
                                                add_benchmark=add_natl_benchmark
                                            />
                                        </div>
                                    </div>
                                </form>
                            </div>
                        </div>

                        // Fixed footer with proper styling and WHITE background
                        <div class="sticky bottom-0 bg-white border-t border-gray-200 px-6 py-4 flex justify-end space-x-3 rounded-b-lg">
                            <button
                                type="button"
                                class="px-4 py-2 bg-gray-100 text-gray-700 rounded border border-gray-300 hover:bg-gray-200 transition-colors text-sm font-medium"
                                on:click=move |_| {
                                    set_show_modal(false);
                                    reset_form();
                                }
                            >
                                "Cancel"
                            </button>
                            <button
                                type="submit"
                                class="px-4 py-2 bg-[#2E3A59] text-white rounded shadow-sm hover:bg-[#1e293b] transition-colors text-sm font-medium"
                                on:click=move |ev| {
                                    ev.prevent_default();
                                    submit_form.dispatch(());
                                }
                            >
                                {move || if editing.get() { "Update Assessment" } else { "Create Assessment" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

// Component for visual sequence display in assessment list
#[component]
fn SequenceVisualization(sequence: Vec<TestSequenceItem>, tests: Vec<Test>) -> impl IntoView {
    let sequence = create_signal(sequence).0;
    let tests = create_signal(tests).0;

    view! {
        <div class="relative">
            <div class="flex flex-wrap gap-4 items-center">
                {move || {
                    let seq = sequence.get();
                    let all_tests = tests.get();

                    seq.iter().enumerate().map(|(index, item)| {
                        let test = all_tests.iter().find(|t| {
                            Uuid::parse_str(&t.test_id).unwrap_or_default() == item.test_id
                        });

                        let (bg_color, border_color, icon) = match item.sequence_behavior {
                            SequenceBehavior::Node => ("bg-blue-50", "border-blue-200", "→"),
                            SequenceBehavior::Attainment => ("bg-green-50", "border-green-200", "✓"),
                            SequenceBehavior::Optional => ("bg-gray-50", "border-gray-200", "?"),
                            SequenceBehavior::Diagnostic => ("bg-purple-50", "border-purple-200", "📊"),
                            SequenceBehavior::Remediation => ("bg-orange-50", "border-orange-200", "🔧"),
                            SequenceBehavior::Branching => ("bg-yellow-50", "border-yellow-200", "⚡"),
                        };

                        view! {
                            <div class="flex items-center">
                                <div class=format!("relative p-3 rounded-lg border-2 {} {} min-w-32", bg_color, border_color)>
                                    <div class="flex items-center space-x-2">
                                        <span class="text-lg">{icon}</span>
                                        <div>
                                            <div class="text-xs font-medium">
                                                {test.map(|t| t.name.clone()).unwrap_or_else(|| "Unknown".to_string())}
                                            </div>
                                            <div class="text-xs text-gray-500">
                                                {format!("{:?}", item.sequence_behavior)}
                                            </div>
                                        </div>
                                    </div>
                                    <div class="absolute -top-2 -left-2 w-6 h-6 bg-[#2E3A59] text-white rounded-full flex items-center justify-center text-xs font-bold">
                                        {item.sequence_order}
                                    </div>
                                </div>

                                // Connection arrow
                                {if index < seq.len() - 1 {
                                    view! {
                                        <div class="mx-2 text-gray-400">
                                            <svg width="24" height="16" viewBox="0 0 24 16" fill="currentColor">
                                                <path d="M16 8l-4-4v3H0v2h12v3l4-4z"/>
                                            </svg>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <div></div> }.into_view()
                                }}
                            </div>
                        }
                    }).collect_view()
                }}
            </div>
        </div>
    }
}

// Advanced sequence builder component
#[component]
fn AdvancedSequenceBuilder(
    test_sequence: ReadSignal<Vec<TestSequenceItem>>,
    set_test_sequence: WriteSignal<Vec<TestSequenceItem>>,
    tests_resource: Resource<(), Result<Vec<Test>, ServerFnError>>,
    sequence_counter: ReadSignal<i32>,
    set_sequence_counter: WriteSignal<i32>,
    dragging_item: ReadSignal<Option<usize>>,
    set_dragging_item: WriteSignal<Option<usize>>,
    show_sequence_details: ReadSignal<Option<usize>>,
    set_show_sequence_details: WriteSignal<Option<usize>>,
    on_drag_start: impl Fn(usize) + 'static + Copy,
    on_drag_over: impl Fn(leptos::ev::DragEvent) + 'static + Copy,
    on_drop: impl Fn(usize) + 'static + Copy,
    connect_tests: impl Fn(usize, Uuid, &str) + 'static + Copy,
    remove_from_sequence: impl Fn(Uuid) + 'static + Copy,
) -> impl IntoView {
    let (selected_test_for_sequence, set_selected_test_for_sequence) =
        create_signal::<Option<Uuid>>(None);
    let (sequence_behavior, set_sequence_behavior) = create_signal(SequenceBehavior::Node);
    let (required_score, set_required_score) = create_signal::<Option<i32>>(None);

    // Enhanced variation management
    let (show_variations_panel, set_show_variations_panel) = create_signal(false);
    let (variation_levels, set_variation_levels) = create_signal::<Vec<VariationLevel>>(vec![]);
    let (editing_variation_index, set_editing_variation_index) =
        create_signal::<Option<usize>>(None);

    // Form fields for adding/editing variations
    let (var_test_id, set_var_test_id) = create_signal::<Option<Uuid>>(None);
    let (var_level, set_var_level) = create_signal(1);
    let (var_description, set_var_description) = create_signal(String::new());
    let (var_required_score, set_var_required_score) = create_signal(60);
    let (var_max_attempts, set_var_max_attempts) = create_signal(2);

    // Helper function to check if a test is a variation
    let is_variation_test = move |test: &Test| -> bool {
        test.name.contains(" - ")
            && (test.name.to_lowercase().contains("practice")
                || test.name.to_lowercase().contains("remedial")
                || test.name.to_lowercase().contains("variation")
                || test.name.to_lowercase().contains("guided")
                || test.comments.to_lowercase().contains("variation:"))
    };

    // Get available variation tests (not already used)
    let get_available_variation_tests = move || -> Vec<Test> {
        let all_tests = tests_resource
            .get()
            .map(|r| r.ok())
            .flatten()
            .unwrap_or_default();
        let current_sequence = test_sequence.get();
        let current_variations = variation_levels.get();

        let used_test_ids: std::collections::HashSet<Uuid> = current_sequence
            .iter()
            .flat_map(|item| {
                let mut ids = vec![item.test_id];
                if let Some(variations) = &item.variation_levels {
                    ids.extend(variations.iter().map(|v| v.test_id));
                }
                ids
            })
            .chain(current_variations.iter().map(|v| v.test_id))
            .collect();

        all_tests
            .into_iter()
            .filter(|test| {
                let test_uuid = Uuid::parse_str(&test.test_id).unwrap_or_default();
                !used_test_ids.contains(&test_uuid) && is_variation_test(test)
            })
            .collect()
    };

    // Reset variation form
    let reset_variation_form = move || {
        set_var_test_id.set(None);
        set_var_level.set(1);
        set_var_description.set(String::new());
        set_var_required_score.set(60);
        set_var_max_attempts.set(2);
        set_editing_variation_index.set(None);
    };

    // Add or update variation
    let save_variation = move |_| {
        if let Some(test_id) = var_test_id.get() {
            let new_variation = VariationLevel {
                level: var_level.get(),
                test_id,
                required_score: Some(var_required_score.get()),
                max_attempts: Some(var_max_attempts.get()),
                description: var_description.get(),
            };

            let mut current_variations = variation_levels.get();

            if let Some(index) = editing_variation_index.get() {
                // Update existing variation
                current_variations[index] = new_variation;
            } else {
                // Add new variation
                current_variations.push(new_variation);
            }

            // Sort by level
            current_variations.sort_by_key(|v| v.level);
            set_variation_levels.set(current_variations);

            // Reset form
            reset_variation_form();
        }
    };

    // Edit existing variation
    let edit_variation = move |index: usize| {
        let variations = variation_levels.get();
        if let Some(variation) = variations.get(index) {
            set_var_test_id.set(Some(variation.test_id));
            set_var_level.set(variation.level);
            set_var_description.set(variation.description.clone());
            set_var_required_score.set(variation.required_score.unwrap_or(60));
            set_var_max_attempts.set(variation.max_attempts.unwrap_or(2));
            set_editing_variation_index.set(Some(index));
        }
    };

    // Remove variation
    let remove_variation = move |index: usize| {
        let mut variations = variation_levels.get();
        variations.remove(index);
        set_variation_levels.set(variations);
        reset_variation_form();
    };

    // Add test to sequence with variations
    let add_test_to_sequence = move |_| {
        if let Some(test_id) = selected_test_for_sequence.get() {
            let order = sequence_counter.get();
            let variations = variation_levels.get();

            let mut new_item = match sequence_behavior.get() {
                SequenceBehavior::Attainment => {
                    let mut item = TestSequenceItem::new_attainment(
                        test_id,
                        order,
                        required_score.get().unwrap_or(70),
                        None,
                        None,
                    );
                    if !variations.is_empty() {
                        item.variation_levels = Some(variations);
                    }
                    item
                }
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
            set_test_sequence.set(current_sequence);
            set_sequence_counter.set(order + 1);

            // Reset form
            set_selected_test_for_sequence.set(None);
            set_sequence_behavior.set(SequenceBehavior::Node);
            set_required_score.set(None);
            set_variation_levels.set(vec![]);
            set_show_variations_panel.set(false);
            reset_variation_form();
        }
    };

    view! {
        <div class="sequence-builder bg-white p-6 rounded-lg border border-gray-200">
            <h4 class="text-gray-900 text-lg font-medium mb-4">"Advanced Visual Sequence Builder"</h4>

            // Enhanced Add Test Panel
            <div class="add-test-panel bg-gray-50 border border-gray-200 rounded-lg p-4 mb-6">
                <h5 class="text-gray-700 font-medium mb-3">"Add Test to Sequence"</h5>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-600 mb-1">"Test"</label>
                        <select
                            class="w-full px-3 py-2 border border-gray-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-blue-500"
                            prop:value={move || selected_test_for_sequence.get().map(|id| id.to_string()).unwrap_or_default()}
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                if value.is_empty() {
                                    set_selected_test_for_sequence.set(None);
                                } else if let Ok(uuid) = Uuid::parse_str(&value) {
                                    set_selected_test_for_sequence.set(Some(uuid));
                                }
                            }
                        >
                            <option value="">"Select a test"</option>
                            {move || {
                                tests_resource.get().map(|tests_result| {
                                    match tests_result {
                                        Ok(tests) => {
                                            let current_sequence = test_sequence.get();
                                            let used_test_ids: Vec<Uuid> = current_sequence.iter()
                                                .flat_map(|item| {
                                                    let mut ids = vec![item.test_id];
                                                    if let Some(variations) = &item.variation_levels {
                                                        ids.extend(variations.iter().map(|v| v.test_id));
                                                    }
                                                    ids
                                                })
                                                .collect();

                                            tests.into_iter()
                                                .filter(|test| {
                                                    let test_uuid = Uuid::parse_str(&test.test_id).unwrap_or_default();
                                                    !used_test_ids.contains(&test_uuid) && !is_variation_test(test)
                                                })
                                                .map(|test| {
                                                    view! {
                                                        <option value=test.test_id.clone() class="text-gray-900">
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
                        <label class="block text-sm font-medium text-gray-600 mb-1">"Behavior"</label>
                        <select
                            class="w-full px-3 py-2 border border-gray-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-blue-500"
                            prop:value={move || format!("{:?}", sequence_behavior.get())}
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                match value.as_str() {
                                    "Node" => set_sequence_behavior.set(SequenceBehavior::Node),
                                    "Attainment" => {
                                        set_sequence_behavior.set(SequenceBehavior::Attainment);
                                        set_show_variations_panel.set(true);
                                    },
                                    "Optional" => set_sequence_behavior.set(SequenceBehavior::Optional),
                                    "Diagnostic" => set_sequence_behavior.set(SequenceBehavior::Diagnostic),
                                    "Remediation" => set_sequence_behavior.set(SequenceBehavior::Remediation),
                                    "Branching" => set_sequence_behavior.set(SequenceBehavior::Branching),
                                    _ => {}
                                }
                            }
                        >
                            <option value="" class="text-gray-900">"None"</option>
                            {SequenceBehavior::iter().map(|behavior| {
                                view! {
                                    <option value=format!("{:?}", behavior) class="text-gray-900">
                                        {format!("{:?}", behavior)}
                                    </option>
                                }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>

                    <Show when=move || matches!(sequence_behavior.get(), SequenceBehavior::Attainment)>
                        <div>
                            <label class="block text-sm font-medium text-gray-600 mb-1">"Required Score"</label>
                            <input
                                type="number"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-blue-500"
                                placeholder="70"
                                prop:value={move || required_score.get().unwrap_or(70)}
                                on:input=move |ev| {
                                    if let Ok(score) = event_target_value(&ev).parse::<i32>() {
                                        set_required_score.set(Some(score));
                                    }
                                }
                            />
                        </div>
                    </Show>
                </div>

                // Enhanced Multi-Level Variations Panel
                <Show when=move || show_variations_panel.get() && matches!(sequence_behavior.get(), SequenceBehavior::Attainment)>
                    <div class="bg-orange-50 border border-orange-200 rounded-lg p-4 mb-4">
                        <div class="flex items-center justify-between mb-3">
                            <h6 class="text-orange-800 font-medium">"Multi-Level Variation Tests (On Fail)"</h6>
                            <button
                                type="button"
                                class="text-orange-600 hover:text-orange-800 text-sm"
                                on:click=move |_| set_show_variations_panel.set(false)
                            >
                                "Hide"
                            </button>
                        </div>
                        <p class="text-xs text-orange-700 mb-4">
                            "Students will progress through these variations vertically if they fail the main test. Maximum 3 levels."
                        </p>

                        // Current Variations List with Visual Preview
                        <div class="mb-4">
                            <div class="text-sm font-medium text-orange-800 mb-2">"Current Variation Stack"</div>
                            <div class="bg-white rounded-lg border border-orange-200 p-3">
                                {move || {
                                    let variations = variation_levels.get();
                                    if variations.is_empty() {
                                        view! {
                                            <div class="text-center text-gray-500 py-4">
                                                <div class="mb-2">"📝"</div>
                                                <div class="text-xs">"No variations added yet"</div>
                                                <div class="text-xs text-gray-400">"Add up to 3 levels below"</div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        let all_tests = tests_resource.get().map(|r| r.ok()).flatten().unwrap_or_default();

                                        view! {
                                            <div class="flex flex-col items-center space-y-3">
                                                {variations.iter().enumerate().map(|(index, variation)| {
                                                    let test = all_tests.iter().find(|t| {
                                                        Uuid::parse_str(&t.test_id).unwrap_or_default() == variation.test_id
                                                    });
                                                    let test_name = test.map(|t| t.name.clone()).unwrap_or_else(|| "Unknown".to_string());

                                                    // Different colors for each level
                                                    let level_color = match variation.level {
                                                        1 => "#fb923c", // orange-400
                                                        2 => "#f97316", // orange-500
                                                        3 => "#ea580c", // orange-600
                                                        _ => "#f97316"
                                                    };

                                                    view! {
                                                        <div class="relative group">
                                                            // Variation Node
                                                            <div
                                                                class="w-16 h-16 rounded-full flex flex-col items-center justify-center text-white font-bold text-xs shadow-md border-2"
                                                                style=format!("background-color: {}; border-color: {}", level_color, level_color)
                                                            >
                                                                <div class="text-sm">"L"{variation.level}</div>
                                                                <div class="text-xs">{variation.required_score.unwrap_or(60)}"%"</div>
                                                            </div>

                                                            // Test name and controls
                                                            <div class="absolute -bottom-12 left-1/2 transform -translate-x-1/2 text-center">
                                                                <div class="text-xs font-medium text-orange-700 whitespace-nowrap max-w-20 truncate">
                                                                    {test_name}
                                                                </div>
                                                                <div class="text-xs text-orange-600">{variation.description.clone()}</div>
                                                            </div>

                                                            // Action buttons (visible on hover)
                                                            <div class="absolute -top-2 -right-2 opacity-0 group-hover:opacity-100 transition-opacity flex space-x-1">
                                                                <button
                                                                    type="button"
                                                                    class="w-5 h-5 bg-blue-500 text-white rounded-full text-xs hover:bg-blue-600 transition-colors"
                                                                    on:click=move |_| edit_variation(index)
                                                                    title="Edit"
                                                                >
                                                                    "✎"
                                                                </button>
                                                                <button
                                                                    type="button"
                                                                    class="w-5 h-5 bg-red-500 text-white rounded-full text-xs hover:bg-red-600 transition-colors"
                                                                    on:click=move |_| remove_variation(index)
                                                                    title="Remove"
                                                                >
                                                                    "×"
                                                                </button>
                                                            </div>
                                                        </div>

                                                        // Vertical Arrow Down (if not last)
                                                        {if index < variations.len() - 1 {
                                                            view! {
                                                                <div class="text-orange-400">
                                                                    <svg width="16" height="20" viewBox="0 0 16 20" fill="currentColor">
                                                                        <path d="M8 18l-4-4h3V2h2v12h3l-4 4z"/>
                                                                    </svg>
                                                                </div>
                                                            }.into_view()
                                                        } else {
                                                            view! {}.into_view()
                                                        }}
                                                    }
                                                }).collect_view()}

                                                // End indicator
                                                {if !variations.is_empty() {
                                                    view! {
                                                        <div class="mt-2 px-3 py-1 bg-gray-100 rounded-full">
                                                            <div class="text-xs text-gray-600">"Teacher Intervention"</div>
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! {}.into_view()
                                                }}
                                            </div>
                                        }.into_view()
                                    }
                                }}
                            </div>
                        </div>

                        // Add/Edit Variation Form
                        <div class="bg-white rounded-lg border border-orange-200 p-4">
                            <div class="text-sm font-medium text-orange-800 mb-3">
                                {move || if editing_variation_index.get().is_some() { "Edit Variation" } else { "Add New Variation" }}
                            </div>

                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                                <div>
                                    <label class="block text-xs font-medium text-orange-700 mb-1">"Variation Test"</label>
                                    <select
                                        class="w-full px-3 py-2 border border-orange-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-orange-500 text-sm"
                                        prop:value={move || var_test_id.get().map(|id| id.to_string()).unwrap_or_default()}
                                        on:change=move |ev| {
                                            let value = event_target_value(&ev);
                                            if value.is_empty() {
                                                set_var_test_id.set(None);
                                            } else if let Ok(uuid) = Uuid::parse_str(&value) {
                                                set_var_test_id.set(Some(uuid));
                                            }
                                        }
                                    >
                                        <option value="">"Select variation test"</option>
                                        {move || {
                                            get_available_variation_tests().into_iter().map(|test| {
                                                view! {
                                                    <option value=test.test_id.clone() class="text-gray-900">
                                                        {test.name.clone()}
                                                    </option>
                                                }
                                            }).collect_view()
                                        }}
                                    </select>
                                </div>

                                <div>
                                    <label class="block text-xs font-medium text-orange-700 mb-1">"Level (1-3)"</label>
                                    <select
                                        class="w-full px-3 py-2 border border-orange-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-orange-500 text-sm"
                                        prop:value={move || var_level.get().to_string()}
                                        on:change=move |ev| {
                                            if let Ok(level) = event_target_value(&ev).parse::<i32>() {
                                                set_var_level.set(level);
                                            }
                                        }
                                    >
                                        <option value="1">"Level 1 (First try)"</option>
                                        <option value="2">"Level 2 (Second try)"</option>
                                        <option value="3">"Level 3 (Final try)"</option>
                                    </select>
                                </div>

                                <div>
                                    <label class="block text-xs font-medium text-orange-700 mb-1">"Description"</label>
                                    <input
                                        type="text"
                                        class="w-full px-3 py-2 border border-orange-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-orange-500 text-sm"
                                        placeholder="e.g., Practice Mode, Guided Version"
                                        prop:value={move || var_description.get()}
                                        on:input=move |ev| set_var_description.set(event_target_value(&ev))
                                    />
                                </div>

                                <div>
                                    <label class="block text-xs font-medium text-orange-700 mb-1">"Required Score %"</label>
                                    <input
                                        type="number"
                                        min="0"
                                        max="100"
                                        class="w-full px-3 py-2 border border-orange-300 rounded-md bg-white text-gray-900 focus:ring-2 focus:ring-orange-500 text-sm"
                                        prop:value={move || var_required_score.get()}
                                        on:input=move |ev| {
                                            if let Ok(score) = event_target_value(&ev).parse::<i32>() {
                                                set_var_required_score.set(score);
                                            }
                                        }
                                    />
                                </div>
                            </div>

                            <div class="flex justify-between items-center">
                                <div class="text-xs text-orange-600">
                                    {move || {
                                        let count = variation_levels.get().len();
                                        if count >= 3 && editing_variation_index.get().is_none() {
                                            "Maximum 3 variations reached".to_string()
                                        } else {
                                            format!("{}/3 variations used", count)
                                        }
                                    }}
                                </div>

                                <div class="flex space-x-2">
                                    <Show when=move || editing_variation_index.get().is_some()>
                                        <button
                                            type="button"
                                            class="px-3 py-1 text-xs bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors"
                                            on:click=move |_| reset_variation_form()
                                        >
                                            "Cancel"
                                        </button>
                                    </Show>

                                    <button
                                        type="button"
                                        class="px-3 py-1 text-xs bg-orange-600 text-white rounded hover:bg-orange-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                        on:click=save_variation
                                        disabled=move || {
                                            let has_test = var_test_id.get().is_some();
                                            let has_description = !var_description.get().trim().is_empty();
                                            let count = variation_levels.get().len();
                                            let is_editing = editing_variation_index.get().is_some();

                                            !has_test || !has_description || (count >= 3 && !is_editing)
                                        }
                                    >
                                        {move || if editing_variation_index.get().is_some() { "Update" } else { "Add Variation" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </Show>

                // Main action button
                <div class="flex justify-end">
                    <button
                        type="button"
                        class="bg-blue-600 text-white px-6 py-2 rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                        on:click=add_test_to_sequence
                        disabled=move || selected_test_for_sequence.get().is_none()
                    >
                        "Add to Sequence"
                    </button>
                </div>
            </div>

            // Enhanced Visual Flow Display
            <EnhancedSequenceFlow
                test_sequence=test_sequence
                tests_resource=tests_resource
                dragging_item=dragging_item
                set_dragging_item=set_dragging_item
                show_sequence_details=show_sequence_details
                set_show_sequence_details=set_show_sequence_details
                remove_from_sequence=remove_from_sequence
                set_test_sequence=set_test_sequence
            />
        </div>
    }
}

// Reusable benchmark section component
#[component]
fn BenchmarkSection(
    benchmarks: ReadSignal<Option<Vec<RangeCategory>>>,
    set_benchmarks: WriteSignal<Option<Vec<RangeCategory>>>,
    benchmark_min: ReadSignal<i32>,
    set_benchmark_min: WriteSignal<i32>,
    benchmark_max: ReadSignal<i32>,
    set_benchmark_max: WriteSignal<i32>,
    benchmark_label: ReadSignal<String>,
    set_benchmark_label: WriteSignal<String>,
    add_benchmark: impl Fn(leptos::ev::MouseEvent) + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            // Existing benchmarks display with improved spacing
            <div class="space-y-3">
                {move || {
                    let benchmarks_list = benchmarks.get().unwrap_or_default();
                    benchmarks_list.into_iter().map(|benchmark| {
                        let benchmark_label_clone = benchmark.label.clone();
                        view! {
                            <div class="flex items-center justify-between bg-gray-50 p-4 rounded-lg border border-gray-200 hover:bg-gray-100 transition-colors">
                                <div class="flex items-center space-x-3 text-gray-900">
                                    <span class="font-medium text-sm min-w-0 flex-shrink-0">{benchmark.label}</span>
                                    <span class="text-gray-400">"|"</span>
                                    <div class="flex items-center space-x-2 text-sm text-gray-700">
                                        <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded text-xs font-medium">{benchmark.min}</span>
                                        <span class="text-gray-500">"to"</span>
                                        <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded text-xs font-medium">{benchmark.max}</span>
                                    </div>
                                </div>
                                <button
                                    type="button"
                                    class="text-sm text-red-600 hover:text-red-800 hover:bg-red-50 px-3 py-1.5 rounded-md transition-colors flex-shrink-0 font-medium"
                                    on:click=move |_| {
                                        let mut current = benchmarks.get().unwrap_or_default();
                                        current.retain(|b| b.label != benchmark_label_clone);
                                        set_benchmarks.set(Some(current));
                                    }
                                >
                                    "Remove"
                                </button>
                            </div>
                        }
                    }).collect_view()
                }}
            </div>

            // Add new benchmark form with improved layout
            <div class="bg-gray-50 p-4 rounded-lg border border-gray-200">
                <h6 class="text-sm font-medium text-gray-700 mb-3">"Add New Benchmark"</h6>
                <div class="grid grid-cols-1 md:grid-cols-4 gap-4 items-end">
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">"Min Score"</label>
                        <input
                            type="number"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-white text-gray-900"
                            placeholder="0"
                            min="0"
                            prop:value={move || benchmark_min.get()}
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    set_benchmark_min.set(v);
                                }
                            }
                        />
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">"Max Score"</label>
                        <input
                            type="number"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-white text-gray-900"
                            placeholder="100"
                            min="0"
                            prop:value={move || benchmark_max.get()}
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    set_benchmark_max.set(v);
                                }
                            }
                        />
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">"Category Label"</label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-white text-gray-900"
                            placeholder="e.g., Mastery, Developing"
                            prop:value={move || benchmark_label.get()}
                            on:input=move |ev| set_benchmark_label.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <button
                            type="button"
                            class="w-full px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md border border-blue-600 hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                            on:click=add_benchmark
                            disabled=move || {
                                let label_empty = benchmark_label.get().trim().is_empty();
                                let invalid_range = benchmark_min.get() >= benchmark_max.get();
                                label_empty || invalid_range
                            }
                        >
                            "Add Benchmark"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SequenceNodeWithVariations(
    seq_item: TestSequenceItem,
    test: Option<Test>,
    all_tests: Vec<Test>,
    index: usize,
    dragging_item: ReadSignal<Option<usize>>,
    set_dragging_item: WriteSignal<Option<usize>>,
    show_sequence_details: ReadSignal<Option<usize>>,
    set_show_sequence_details: WriteSignal<Option<usize>>,
    remove_from_sequence: impl Fn(Uuid) + 'static + Copy,
    test_sequence: ReadSignal<Vec<TestSequenceItem>>,
    set_test_sequence: WriteSignal<Vec<TestSequenceItem>>,
) -> impl IntoView {
    let item_test_id = seq_item.test_id;
    let has_variations = seq_item
        .variation_levels
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    // Clone the variations to avoid lifetime issues
    let variations = seq_item.variation_levels.clone().unwrap_or_default();
    let main_test_id = seq_item.test_id; // For closure capture

    let (node_color, icon, border_color) = match seq_item.sequence_behavior {
        SequenceBehavior::Node => ("#3b82f6", "→", "#2563eb"),
        SequenceBehavior::Attainment => ("#10b981", "✓", "#059669"),
        SequenceBehavior::Optional => ("#6b7280", "?", "#4b5563"),
        SequenceBehavior::Diagnostic => ("#8b5cf6", "📊", "#7c3aed"),
        SequenceBehavior::Remediation => ("#f59e0b", "🔧", "#d97706"),
        SequenceBehavior::Branching => ("#eab308", "⚡", "#ca8a04"),
    };

    let test_name = test
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let short_name = if test_name.len() > 12 {
        format!("{}...", &test_name[0..9])
    } else {
        test_name.clone()
    };

    view! {
        <div class="flex flex-col items-center">
            // Main Test Node
            <div
                class="sequence-node relative group cursor-move transition-all duration-200"
                class:opacity-50={move || dragging_item.get() == Some(index)}
                draggable="true"
                on:dragstart=move |ev| {
                    ev.stop_propagation();
                    if let Some(dt) = ev.data_transfer() {
                        let _ = dt.set_data("text/plain", &index.to_string());
                        let _ = dt.set_effect_allowed("move");
                    }
                    set_dragging_item.set(Some(index));
                }
                on:dragover=move |ev| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    if let Some(dt) = ev.data_transfer() {
                        let _ = dt.set_drop_effect("move");
                    }
                }
                on:drop=move |ev| {
                    ev.prevent_default();
                    ev.stop_propagation();

                    if let Some(dt) = ev.data_transfer() {
                        if let Ok(data) = dt.get_data("text/plain") {
                            if let Ok(source_index) = data.parse::<usize>() {
                                if source_index != index {
                                    let mut sequence = test_sequence.get();
                                    if source_index < sequence.len() && index < sequence.len() {
                                        let item = sequence.remove(source_index);
                                        sequence.insert(index, item);

                                        for (idx, item) in sequence.iter_mut().enumerate() {
                                            item.sequence_order = (idx + 1) as i32;
                                        }

                                        set_test_sequence.set(sequence);
                                    }
                                }
                            }
                        }
                    }
                    set_dragging_item.set(None);
                }
                on:dragend=move |_| {
                    set_dragging_item.set(None);
                }
            >
                // Main Node Circle
                <div
                    class="w-20 h-20 rounded-full flex flex-col items-center justify-center text-white font-bold text-sm shadow-lg hover:scale-110 transition-transform duration-200"
                    style=format!("background-color: {}; border: 3px solid {}", node_color, border_color)
                >
                    <div class="text-lg">{icon}</div>
                    <div class="text-xs">{seq_item.sequence_order}</div>
                </div>

                // Node Info
                <div class="absolute -bottom-12 left-1/2 transform -translate-x-1/2 text-center">
                    <div class="text-xs font-medium text-gray-700 whitespace-nowrap">{short_name}</div>
                    <div class="text-xs text-gray-500">{format!("{:?}", seq_item.sequence_behavior)}</div>
                    {if has_variations {
                        view! {
                            <div class="text-xs text-orange-600 font-medium">
                                {format!("{} Variations", variations.len())}
                            </div>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                </div>

                // Actions (visible on hover)
                <div class="absolute -top-2 -right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                        type="button"
                        class="w-6 h-6 bg-red-500 text-white rounded-full text-xs hover:bg-red-600 transition-colors"
                        on:click=move |ev| {
                            ev.stop_propagation();
                            remove_from_sequence(item_test_id);
                        }
                        title="Remove"
                    >
                        "×"
                    </button>
                </div>

                // Details toggle
                <div class="absolute -bottom-20 left-1/2 transform -translate-x-1/2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                        type="button"
                        class="text-xs bg-gray-100 text-gray-700 px-2 py-1 rounded hover:bg-gray-200 transition-colors"
                        on:click=move |ev| {
                            ev.stop_propagation();
                            if show_sequence_details.get() == Some(index) {
                                set_show_sequence_details.set(None);
                            } else {
                                set_show_sequence_details.set(Some(index));
                            }
                        }
                    >
                        "Details"
                    </button>
                </div>
            </div>

            // Vertical Variation Stack (if has variations)
            {if has_variations {
                view! {
                    <div class="mt-6 flex flex-col items-center space-y-3">
                        // Vertical Arrow Down
                        <div class="text-orange-500">
                            <svg width="16" height="24" viewBox="0 0 16 24" fill="currentColor">
                                <path d="M8 20l-4-4h3V0h2v16h3l-4 4z"/>
                            </svg>
                        </div>

                        // Variation Levels
                        {variations.iter().enumerate().map(|(var_index, variation)| {
                            let var_test = all_tests.iter().find(|t| {
                                Uuid::parse_str(&t.test_id).unwrap_or_default() == variation.test_id
                            });

                            let var_name = var_test.map(|t| t.name.clone()).unwrap_or_else(|| "Unknown".to_string());
                            let var_short_name = if var_name.len() > 10 {
                                format!("{}...", &var_name[0..7])
                            } else {
                                var_name.clone()
                            };

                            // Different shades of orange for each level
                            let var_color = match variation.level {
                                1 => "#fb923c", // orange-400
                                2 => "#f97316", // orange-500
                                3 => "#ea580c", // orange-600
                                _ => "#f97316"
                            };

                            let variation_test_id = variation.test_id;

                            view! {
                                <div class="relative">
                                    // Variation Node (smaller than main node)
                                    <div
                                        class="w-14 h-14 rounded-full flex flex-col items-center justify-center text-white font-bold text-xs shadow-md hover:scale-105 transition-transform duration-200"
                                        style=format!("background-color: {}; border: 2px solid {}", var_color, var_color)
                                        title=format!("Level {} Variation: {}", variation.level, var_name)
                                    >
                                        <div class="text-sm">"L"{variation.level}</div>
                                        <div class="text-xs">{variation.level}</div>
                                    </div>

                                    // Variation Info
                                    <div class="absolute -bottom-8 left-1/2 transform -translate-x-1/2 text-center">
                                        <div class="text-xs font-medium text-orange-700 whitespace-nowrap">{var_short_name}</div>
                                        <div class="text-xs text-orange-600">"Level " {variation.level}</div>
                                    </div>

                                    // Remove variation button (on hover)
                                    <div class="absolute -top-1 -right-1 opacity-0 hover:opacity-100 transition-opacity">
                                        <button
                                            type="button"
                                            class="w-4 h-4 bg-red-500 text-white rounded-full text-xs hover:bg-red-600 transition-colors"
                                            on:click=move |ev| {
                                                ev.stop_propagation();
                                                // Remove this variation from the sequence item
                                                let mut sequence = test_sequence.get();
                                                if let Some(item) = sequence.iter_mut().find(|item| item.test_id == main_test_id) {
                                                    if let Some(ref mut variations) = item.variation_levels {
                                                        variations.retain(|v| v.test_id != variation_test_id);
                                                        if variations.is_empty() {
                                                            item.variation_levels = None;
                                                        }
                                                    }
                                                }
                                                set_test_sequence.set(sequence);
                                            }
                                            title="Remove variation"
                                        >
                                            "×"
                                        </button>
                                    </div>
                                </div>

                                // Vertical Arrow Down (if not last variation)
                                {if var_index < variations.len() - 1 {
                                    view! {
                                        <div class="text-orange-400 mt-2">
                                            <svg width="12" height="16" viewBox="0 0 12 16" fill="currentColor">
                                                <path d="M6 14l-3-3h2V0h2v11h2l-3 3z"/>
                                            </svg>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {}.into_view()
                                }}
                            }
                        }).collect_view()}

                        // "End of Variations" indicator
                        <div class="mt-3 px-3 py-1 bg-gray-100 rounded-full">
                            <div class="text-xs text-gray-600">"End Remediation"</div>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

#[component]
fn EnhancedSequenceFlow(
    test_sequence: ReadSignal<Vec<TestSequenceItem>>,
    tests_resource: Resource<(), Result<Vec<Test>, ServerFnError>>,
    dragging_item: ReadSignal<Option<usize>>,
    set_dragging_item: WriteSignal<Option<usize>>,
    show_sequence_details: ReadSignal<Option<usize>>,
    set_show_sequence_details: WriteSignal<Option<usize>>,
    remove_from_sequence: impl Fn(Uuid) + 'static + Copy,
    set_test_sequence: WriteSignal<Vec<TestSequenceItem>>,
) -> impl IntoView {
    view! {
        <div class="sequence-flow-container bg-white border-2 border-dashed border-gray-300 rounded-lg p-6 min-h-96">
            <div class="flex items-center justify-between mb-6">
                <h5 class="text-gray-700 font-medium">"Visual Sequence Flow with Vertical Variation Stacks"</h5>
                <div class="text-sm text-gray-500">
                    {move || {
                        let count = test_sequence.get().len();
                        format!("{} test{} in sequence", count, if count == 1 { "" } else { "s" })
                    }}
                </div>
            </div>

            {move || {
                let sequence = test_sequence.get();
                if sequence.is_empty() {
                    view! {
                        <div class="flex items-center justify-center h-48 text-gray-500 text-sm">
                            <div class="text-center">
                                <div class="mb-4 text-4xl">"🔄"</div>
                                <div class="text-lg font-medium mb-2">"No tests in sequence yet"</div>
                                <div class="text-gray-400">"Add tests above to build your assessment flow"</div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    let all_tests = tests_resource.get().map(|r| r.ok()).flatten().unwrap_or_default();

                    view! {
                        // Responsive grid layout for better scaling
                        <div class="overflow-x-auto pb-4">
                            <div
                                class="flex items-start gap-8 min-w-fit"
                                style="min-width: max-content;"
                            >
                                {sequence.iter().enumerate().map(|(index, seq_item)| {
                                    view! {
                                        <div class="flex items-start shrink-0">
                                            // Main Test Node with Improved Spacing
                                            <ImprovedSequenceNodeWithVariations
                                                seq_item=seq_item.clone()
                                                all_tests=all_tests.clone()
                                                index=index
                                                dragging_item=dragging_item
                                                set_dragging_item=set_dragging_item
                                                show_sequence_details=show_sequence_details
                                                set_show_sequence_details=set_show_sequence_details
                                                remove_from_sequence=remove_from_sequence
                                                test_sequence=test_sequence
                                                set_test_sequence=set_test_sequence
                                            />

                                            // Horizontal Connection Arrow (if not last item)
                                            {if index < sequence.len() - 1 {
                                                view! {
                                                    <div class="flex items-center justify-center h-20 mx-4 mt-16">
                                                        <div class="flex flex-col items-center">
                                                            <div class="bg-blue-100 text-blue-700 px-2 py-1 rounded-full text-xs font-medium mb-1">
                                                                "PASS"
                                                            </div>
                                                            <svg width="32" height="16" viewBox="0 0 32 16" fill="currentColor" class="text-blue-500">
                                                                <path d="M24 8l-4-4v3H4v2h16v3l4-4z"/>
                                                            </svg>
                                                        </div>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div class="flex items-center justify-center h-20 mx-4 mt-16">
                                                        <div class="bg-green-100 text-green-700 px-3 py-2 rounded-full text-sm font-medium">
                                                            "🎯 COMPLETE"
                                                        </div>
                                                    </div>
                                                }.into_view()
                                            }}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        // Sequence Controls for 10+ items
                        <div class="mt-6 flex justify-between items-center bg-gray-50 rounded-lg p-4">
                            <div class="text-sm text-gray-600">
                                "Drag tests to reorder • Click info button for details • Remove with × button"
                            </div>
                            <div class="flex items-center space-x-2">
                                <button
                                    type="button"
                                    class="text-xs bg-blue-100 text-blue-700 px-3 py-1 rounded-full hover:bg-blue-200 transition-colors"
                                    on:click=move |_| {
                                        // Could implement collapse/expand all
                                        set_show_sequence_details.set(None);
                                    }
                                >
                                    "Collapse All Details"
                                </button>
                                <span class="text-gray-300">"|"</span>
                                <div class="text-xs text-gray-500">
                                    "Scroll horizontally to see all tests →"
                                </div>
                            </div>
                        </div>
                    }.into_view()
                }
            }}

            // Enhanced Legend with better organization
            <div class="mt-8 grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div class="bg-blue-50 rounded-lg border border-blue-200 p-4">
                    <h6 class="text-sm font-semibold mb-3 text-blue-900">"Test Behaviors"</h6>
                    <div class="space-y-2">
                        <div class="flex items-center text-xs text-blue-800">
                            <div class="w-4 h-4 rounded-full bg-green-500 mr-3"></div>
                            <div class="flex-1">
                                <span class="font-medium">"Attainment (✓)"</span>
                                <div class="text-blue-600">"Requires specific score to pass"</div>
                            </div>
                        </div>
                        <div class="flex items-center text-xs text-blue-800">
                            <div class="w-4 h-4 rounded-full bg-blue-500 mr-3"></div>
                            <div class="flex-1">
                                <span class="font-medium">"Node (→)"</span>
                                <div class="text-blue-600">"Simple progression test"</div>
                            </div>
                        </div>
                        <div class="flex items-center text-xs text-blue-800">
                            <div class="w-4 h-4 rounded-full bg-gray-500 mr-3"></div>
                            <div class="flex-1">
                                <span class="font-medium">"Optional (?)"</span>
                                <div class="text-blue-600">"Can be skipped"</div>
                            </div>
                        </div>
                        <div class="flex items-center text-xs text-blue-800">
                            <div class="w-4 h-4 rounded-full bg-purple-500 mr-3"></div>
                            <div class="flex-1">
                                <span class="font-medium">"Diagnostic (📊)"</span>
                                <div class="text-blue-600">"Assessment only"</div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="bg-orange-50 rounded-lg border border-orange-200 p-4">
                    <h6 class="text-sm font-semibold mb-3 text-orange-900">"Variation System"</h6>
                    <div class="space-y-2">
                        <div class="flex items-center text-xs text-orange-800">
                            <div class="w-4 h-4 rounded-full bg-orange-400 mr-3"></div>
                            <div class="flex-1">
                                <span class="font-medium">"Level 1"</span>
                                <div class="text-orange-600">"First remediation attempt"</div>
                            </div>
                        </div>
                        <div class="flex items-center text-xs text-orange-800">
                            <div class="w-4 h-4 rounded-full bg-orange-500 mr-3"></div>
                            <div class="flex-1">
                                <span class="font-medium">"Level 2"</span>
                                <div class="text-orange-600">"Second remediation attempt"</div>
                            </div>
                        </div>
                        <div class="flex items-center text-xs text-orange-800">
                            <div class="w-4 h-4 rounded-full bg-orange-600 mr-3"></div>
                            <div class="flex-1">
                                <span class="font-medium">"Level 3"</span>
                                <div class="text-orange-600">"Final remediation attempt"</div>
                            </div>
                        </div>
                        <div class="mt-3 p-2 bg-orange-100 rounded text-xs text-orange-800">
                            <span class="font-medium">"Flow:"</span>
                            " Student fails main → L1 → L2 → L3 → Teacher intervention"
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ImprovedSequenceNodeWithVariations(
    seq_item: TestSequenceItem,
    all_tests: Vec<Test>,
    index: usize,
    dragging_item: ReadSignal<Option<usize>>,
    set_dragging_item: WriteSignal<Option<usize>>,
    show_sequence_details: ReadSignal<Option<usize>>,
    set_show_sequence_details: WriteSignal<Option<usize>>,
    remove_from_sequence: impl Fn(Uuid) + 'static + Copy,
    test_sequence: ReadSignal<Vec<TestSequenceItem>>,
    set_test_sequence: WriteSignal<Vec<TestSequenceItem>>,
) -> impl IntoView {
    let item_test_id = seq_item.test_id;
    let has_variations = seq_item
        .variation_levels
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    let variations = seq_item.variation_levels.clone().unwrap_or_default();
    let main_test_id = seq_item.test_id;

    let (node_color, icon, border_color, behavior_name) = match seq_item.sequence_behavior {
        SequenceBehavior::Node => ("#3b82f6", "→", "#2563eb", "Node"),
        SequenceBehavior::Attainment => ("#10b981", "✓", "#059669", "Attainment"),
        SequenceBehavior::Optional => ("#6b7280", "?", "#4b5563", "Optional"),
        SequenceBehavior::Diagnostic => ("#8b5cf6", "📊", "#7c3aed", "Diagnostic"),
        SequenceBehavior::Remediation => ("#f59e0b", "🔧", "#d97706", "Remediation"),
        SequenceBehavior::Branching => ("#eab308", "⚡", "#ca8a04", "Branching"),
    };

    let test = all_tests
        .iter()
        .find(|t| Uuid::parse_str(&t.test_id).unwrap_or_default() == item_test_id);

    let test_name = test
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let short_name = if test_name.len() > 16 {
        format!("{}...", &test_name[0..13])
    } else {
        test_name.clone()
    };

    let test_name_for_details = test_name.clone();

    // Remove individual variation from a test
    let remove_variation_from_test = move |variation_test_id: Uuid| {
        let mut sequence = test_sequence.get();
        if let Some(item) = sequence
            .iter_mut()
            .find(|item| item.test_id == main_test_id)
        {
            if let Some(ref mut variations) = item.variation_levels {
                variations.retain(|v| v.test_id != variation_test_id);
                if variations.is_empty() {
                    item.variation_levels = None;
                }
            }
        }
        set_test_sequence.set(sequence);
    };

    view! {
        // Container with controlled width and proper spacing
        <div class="flex flex-col items-center w-48 min-h-96">
            // Main Test Node with improved positioning
            <div
                class="sequence-node relative group cursor-move transition-all duration-200 hover:scale-105 z-10"
                class:opacity-50={move || dragging_item.get() == Some(index)}
                draggable="true"
                on:dragstart=move |ev| {
                    ev.stop_propagation();
                    if let Some(dt) = ev.data_transfer() {
                        let _ = dt.set_data("text/plain", &index.to_string());
                        let _ = dt.set_effect_allowed("move");
                    }
                    set_dragging_item.set(Some(index));
                }
                on:dragover=move |ev| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    if let Some(dt) = ev.data_transfer() {
                        let _ = dt.set_drop_effect("move");
                    }
                }
                on:drop=move |ev| {
                    ev.prevent_default();
                    ev.stop_propagation();

                    if let Some(dt) = ev.data_transfer() {
                        if let Ok(data) = dt.get_data("text/plain") {
                            if let Ok(source_index) = data.parse::<usize>() {
                                if source_index != index {
                                    let mut sequence = test_sequence.get();
                                    if source_index < sequence.len() && index < sequence.len() {
                                        let item = sequence.remove(source_index);
                                        sequence.insert(index, item);

                                        for (idx, item) in sequence.iter_mut().enumerate() {
                                            item.sequence_order = (idx + 1) as i32;
                                        }

                                        set_test_sequence.set(sequence);
                                    }
                                }
                            }
                        }
                    }
                    set_dragging_item.set(None);
                }
                on:dragend=move |_| {
                    set_dragging_item.set(None);
                }
            >
                // Main Node Circle with consistent sizing
                <div
                    class="w-20 h-20 rounded-full flex flex-col items-center justify-center text-white font-bold shadow-xl hover:shadow-2xl transition-shadow duration-200 relative"
                    style=format!("background: linear-gradient(135deg, {}, {}); border: 3px solid {}", node_color, node_color, border_color)
                >
                    <div class="text-lg mb-1">{icon}</div>
                    <div class="text-xs font-bold bg-black bg-opacity-20 rounded-full w-5 h-5 flex items-center justify-center">
                        {seq_item.sequence_order}
                    </div>
                </div>

                // Enhanced Actions (improved positioning)
                <div class="absolute -top-2 -right-2 opacity-0 group-hover:opacity-100 transition-opacity flex space-x-1 z-20">
                    <button
                        type="button"
                        class="w-6 h-6 bg-blue-500 text-white rounded-full text-xs hover:bg-blue-600 transition-colors shadow-md"
                        on:click=move |ev| {
                            ev.stop_propagation();
                            if show_sequence_details.get() == Some(index) {
                                set_show_sequence_details.set(None);
                            } else {
                                set_show_sequence_details.set(Some(index));
                            }
                        }
                        title="View details"
                    >
                        "ⓘ"
                    </button>
                    <button
                        type="button"
                        class="w-6 h-6 bg-red-500 text-white rounded-full text-xs hover:bg-red-600 transition-colors shadow-md"
                        on:click=move |ev| {
                            ev.stop_propagation();
                            remove_from_sequence(item_test_id);
                        }
                        title="Remove test"
                    >
                        "×"
                    </button>
                </div>
            </div>

            // Improved Node Info with better spacing
            <div class="mt-4 text-center w-full px-2">
                <div class="text-sm font-semibold text-gray-800 truncate mb-1" title={test_name.clone()}>
                    {short_name}
                </div>
                <div class="text-xs text-gray-600 bg-gray-100 rounded-full px-2 py-1 mb-1">
                    {behavior_name}
                </div>
                {if seq_item.sequence_behavior == SequenceBehavior::Attainment {
                    view! {
                        <div class="text-xs text-green-700 font-medium bg-green-100 rounded-full px-2 py-1 mb-1">
                            "Requires "{seq_item.required_score.unwrap_or(70)}"%"
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}
                {if has_variations {
                    view! {
                        <div class="text-xs text-orange-600 font-bold bg-orange-100 rounded-full px-2 py-1">
                            {format!("{} Variation{}", variations.len(), if variations.len() == 1 { "" } else { "s" })}
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}
            </div>

            // "ON FAIL" Label and Arrow Down (improved spacing)
            {if has_variations {
                view! {
                    <div class="mt-6 flex flex-col items-center">
                        <div class="bg-red-100 text-red-700 px-3 py-1 rounded-full text-xs font-bold border border-red-200 mb-3">
                            "ON FAIL ↓"
                        </div>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="h-12"></div> // Spacer to maintain consistent height
                }.into_view()
            }}

            // Enhanced Vertical Variation Stack with proper spacing
            {if has_variations {
                view! {
                    <div class="flex flex-col items-center space-y-6 w-full">
                        {variations.iter().enumerate().map(|(var_index, variation)| {
                            let var_test = all_tests.iter().find(|t| {
                                Uuid::parse_str(&t.test_id).unwrap_or_default() == variation.test_id
                            });

                            let var_name = var_test.map(|t| t.name.clone()).unwrap_or_else(|| "Unknown".to_string());
                            let var_short_name = if var_name.len() > 14 {
                                format!("{}...", &var_name[0..11])
                            } else {
                                var_name.clone()
                            };

                            // Enhanced level colors
                            let (var_color, var_border) = match variation.level {
                                1 => ("#fb923c", "#ea580c"),
                                2 => ("#f97316", "#c2410c"),
                                3 => ("#ea580c", "#9a3412"),
                                _ => ("#f97316", "#c2410c")
                            };

                            let variation_test_id = variation.test_id;

                            view! {
                                <div class="relative group w-full flex flex-col items-center">
                                    // Enhanced Variation Node
                                    <div
                                        class="w-16 h-16 rounded-full flex flex-col items-center justify-center text-white font-bold shadow-lg hover:shadow-xl transition-all duration-200 hover:scale-105 relative"
                                        style=format!("background: linear-gradient(135deg, {}, {}); border: 2px solid {}", var_color, var_border, var_border)
                                        title=format!("Level {} Variation: {} ({}% required)", variation.level, var_name, variation.required_score.unwrap_or(60))
                                    >
                                        <div class="text-sm font-bold">"L"{variation.level}</div>
                                        <div class="text-xs bg-black bg-opacity-20 rounded px-1">
                                            {variation.required_score.unwrap_or(60)}"%"
                                        </div>
                                    </div>

                                    // Enhanced Variation Info with better positioning
                                    <div class="mt-3 text-center w-full px-2">
                                        <div class="text-xs font-semibold text-orange-800 truncate mb-1" title={var_name.clone()}>
                                            {var_short_name}
                                        </div>
                                        <div class="text-xs text-orange-600 bg-orange-100 rounded-full px-2 py-1 mb-1">
                                            {variation.description.clone()}
                                        </div>
                                        <div class="text-xs text-orange-700 font-medium">
                                            "Level "{variation.level}" • "{variation.max_attempts.unwrap_or(2)}" attempts"
                                        </div>
                                    </div>

                                    // Action button (improved positioning)
                                    <div class="absolute -top-1 -right-1 opacity-0 group-hover:opacity-100 transition-opacity z-10">
                                        <button
                                            type="button"
                                            class="w-5 h-5 bg-red-500 text-white rounded-full text-xs hover:bg-red-600 transition-colors shadow-md"
                                            on:click=move |ev| {
                                                ev.stop_propagation();
                                                remove_variation_from_test(variation_test_id);
                                            }
                                            title="Remove this variation"
                                        >
                                            "×"
                                        </button>
                                    </div>

                                    // Enhanced flow indicator (if not last variation)
                                    {if var_index < variations.len() - 1 {
                                        view! {
                                            <div class="flex flex-col items-center mt-4">
                                                <div class="bg-red-100 text-red-600 px-2 py-1 rounded-full text-xs font-medium border border-red-200 mb-2">
                                                    "STILL FAIL ↓"
                                                </div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! {}.into_view()
                                    }}
                                </div>
                            }
                        }).collect_view()}

                        // Enhanced "End of Variations" indicator
                        <div class="mt-6 flex flex-col items-center">
                            <div class="bg-purple-100 text-purple-700 px-4 py-2 rounded-lg text-xs font-bold border border-purple-200 mb-2 text-center">
                                "🚨 TEACHER INTERVENTION"
                            </div>
                            <div class="text-xs text-gray-600 text-center max-w-40">
                                "All remediation attempts exhausted"
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! {}.into_view()
            }}

            // Detailed Information Panel (improved positioning and spacing)
            <Show when=move || show_sequence_details.get() == Some(index)>
                <div class="mt-8 bg-white border border-gray-200 rounded-lg shadow-xl p-4 w-full max-w-sm z-30 relative">
                    <h6 class="font-semibold text-gray-800 mb-3 text-center">"Test Details"</h6>
                    <div class="space-y-3 text-sm">
                        <div class="bg-gray-50 p-2 rounded">
                            <div class="font-medium text-gray-700">"Name:"</div>
                            <div class="text-gray-900">{test_name_for_details.clone()}</div>
                        </div>
                        <div class="grid grid-cols-2 gap-2 text-xs">
                            <div class="bg-gray-50 p-2 rounded">
                                <div class="font-medium text-gray-700">"Behavior:"</div>
                                <div class="text-gray-900">{behavior_name}</div>
                            </div>
                            <div class="bg-gray-50 p-2 rounded">
                                <div class="font-medium text-gray-700">"Order:"</div>
                                <div class="text-gray-900">{seq_item.sequence_order}</div>
                            </div>
                        </div>
                        {if seq_item.sequence_behavior == SequenceBehavior::Attainment {
                            view! {
                                <div class="bg-green-50 p-2 rounded text-xs">
                                    <div class="font-medium text-green-700">"Required Score:"</div>
                                    <div class="text-green-900">{seq_item.required_score.unwrap_or(70)}"%"</div>
                                </div>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
                        <div class="bg-gray-50 p-2 rounded text-xs">
                            <div class="font-medium text-gray-700">"Max Attempts:"</div>
                            <div class="text-gray-900">{seq_item.max_attempts.unwrap_or(1)}</div>
                        </div>
                        {if has_variations {
                            let variations_clone = variations.clone();
                            let all_tests_clone = all_tests.clone();
                            view! {
                                <div class="bg-orange-50 p-2 rounded text-xs">
                                    <div class="font-medium text-orange-700 mb-2">"Variations:"</div>
                                    <div class="space-y-1">
                                        {variations_clone.iter().map(|var| {
                                            let var_test_name = all_tests_clone.iter()
                                                .find(|t| Uuid::parse_str(&t.test_id).unwrap_or_default() == var.test_id)
                                                .map(|t| t.name.clone())
                                                .unwrap_or_else(|| "Unknown".to_string());

                                            view! {
                                                <div class="flex justify-between items-center bg-white p-2 rounded border">
                                                    <div>
                                                        <div class="font-medium">"Level "{var.level}</div>
                                                        <div class="text-gray-600">{var_test_name}</div>
                                                    </div>
                                                    <div class="text-orange-600 font-bold">{var.required_score.unwrap_or(60)}"%"</div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
                    </div>
                    <div class="mt-4 text-center">
                        <button
                            type="button"
                            class="text-xs bg-gray-100 text-gray-700 px-3 py-1 rounded hover:bg-gray-200 transition-colors"
                            on:click=move |_| set_show_sequence_details.set(None)
                        >
                            "Close Details"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
