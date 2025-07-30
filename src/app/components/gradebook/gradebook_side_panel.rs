use crate::app::components::data_processing::{
    AssessmentSummary, Progress, StudentResultsSummary, TestDetail,
};
use crate::app::components::enhanced_login_form::{
    use_student_mapping_service, DeAnonymizedStudent, StudentMappingService,
};
use crate::app::components::test_item::GenericTestModal;
use crate::app::middleware::global_settings::use_settings;
use crate::app::models::student::Student;
use chrono::prelude::*;
use icondata::{
    BiXCircleRegular, HiUserCircleOutlineLg, RiArrowRightSArrowsLine, RiBookmarkBusinessLine,
    RiFilePaper2DocumentLine,
};
use leptos::*;
use leptos_icons::Icon;
use leptos_router::*;

#[derive(Clone, PartialEq)]
pub enum ScorePanelType {
    AssessmentScore(String),     // Assessment ID
    TestScore(String, i32, i32), // Test ID, Student ID
    None,
}

#[component]
pub fn StudentScorePanel(
    #[prop(into)] show: Signal<bool>,
    #[prop(into)] panel_type: Signal<ScorePanelType>,
    #[prop(into)] set_show: Callback<bool>,
    #[prop(into)] student: Signal<Option<Student>>,
    #[prop(into)] assessment_data: Signal<Option<AssessmentSummary>>,
    #[prop(into)] test_data: Signal<Option<TestDetail>>,
    #[prop(into)] next_test: Signal<Option<String>>, // Next test ID in sequence
) -> impl IntoView {
    // Get global settings and student mapping service for de-anonymization
    let (settings, _) = use_settings();
    let (student_mapping_service, _) = use_student_mapping_service();

    let anonymization_enabled = move || settings.get().student_protections;

    // Format date for display
    let format_date = |date: DateTime<Utc>| date.format("%B %d, %Y").to_string();

    // Generate progress color based on status
    let get_progress_color = |progress: &Progress| match progress {
        Progress::Completed => "bg-green-100 text-green-800",
        Progress::Ongoing => "bg-yellow-100 text-yellow-800",
        Progress::NotStarted => "bg-blue-100 text-blue-800",
    };

    // Calculate percentage for progress bars
    let calculate_percentage = move |score: i32, total: i32| -> i32 {
        if total == 0 {
            0
        } else {
            (score as f32 / total as f32 * 100.0) as i32
        }
    };

    // Helper function to get student display name with de-anonymization support
    let get_student_display_name = move |student: &Student| -> String {
        if anonymization_enabled() {
            if let Some(service) = student_mapping_service.get() {
                if let Some(mapping) = service.get_original_student_info(student.student_id) {
                    return format!("{} {}", mapping.firstname, mapping.lastname);
                }
            }
        }

        // Fallback to anonymized or regular display
        let firstname = student.firstname.as_deref().unwrap_or("Student");
        let lastname = student
            .lastname
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("#{}", student.student_id));
        format!("{} {}", firstname, lastname)
    };

    // Helper function to get student ID display with de-anonymization support
    let get_student_display_id = move |student: &Student| -> String {
        if anonymization_enabled() {
            if let Some(service) = student_mapping_service.get() {
                if let Some(mapping) = service.get_original_student_info(student.student_id) {
                    return mapping.original_student_id.to_string();
                }
            }
        }

        // Fallback to current student ID
        student.student_id.to_string()
    };

    // Close panel function
    let close_panel = move |_| {
        set_show.call(false);
    };

    view! {
        <div
            class="fixed right-0 top-16 h-screen w-96 bg-white shadow-lg transform transition-transform duration-300 z-50 border-l border-gray-200 flex flex-col"
            class=("translate-x-full", move || !show.get())
        >
            <div class="flex justify-between items-center p-4 border-b border-gray-200 bg-[#DADADA] flex-shrink-0">
                <h2 class="text-lg font-bold text-[#2E3A59]">
                    {move || {
                        match panel_type.get() {
                            ScorePanelType::AssessmentScore(_) => "Assessment Details",
                            ScorePanelType::TestScore(_, _, _) => "Test Score Details",
                            ScorePanelType::None => ""
                        }
                    }}
                </h2>
                <button
                    class="text-gray-500 hover:text-gray-700 focus:outline-none"
                    on:click=close_panel
                >
                    <Icon icon=BiXCircleRegular class="w-6 h-6" />
                </button>
            </div>

            <div class="p-4 overflow-y-auto flex-grow">
                {move || {
                    if !show.get() {
                        view! { <div></div> }.into_view()
                    } else {
                        // Student info section with de-anonymization support
                        let student_info = view! {
                            <div class="mb-6 bg-[#F9F9F8] p-4 rounded-lg shadow-sm">
                                <div class="flex items-center mb-3">
                                    <Icon
                                        icon=HiUserCircleOutlineLg
                                        class="w-12 h-12 text-[#2E3A59] mr-3"
                                    />
                                    <div>
                                        <h3 class="text-lg font-semibold text-[#2E3A59]">
                                            {move || {
                                                student.get()
                                                    .map(|s| get_student_display_name(&s))
                                                    .unwrap_or_else(|| "Unknown Student".to_string())
                                            }}
                                        </h3>
                                        <div class="text-sm text-gray-600">
                                            <p>
                                                {move || {
                                                    if anonymization_enabled() && student_mapping_service.get().is_some() {
                                                        "Original ID: "
                                                    } else {
                                                        "ID: "
                                                    }
                                                }}
                                                {move || {
                                                    student.get()
                                                        .map(|s| get_student_display_id(&s))
                                                        .unwrap_or_else(|| "N/A".to_string())
                                                }}
                                            </p>
                                            // Show anonymization status if enabled
                                            {move || {
                                                if anonymization_enabled() {
                                                    if student_mapping_service.get().is_some() {
                                                        view! {
                                                            <p class="text-green-600 text-xs mt-1">
                                                                "✓ De-anonymized data"
                                                            </p>
                                                        }.into_view()
                                                    } else {
                                                        view! {
                                                            <p class="text-yellow-600 text-xs mt-1">
                                                                "⚠ Anonymized data"
                                                            </p>
                                                        }.into_view()
                                                    }
                                                } else {
                                                    view! { <span></span> }.into_view()
                                                }
                                            }}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        };

                        match panel_type.get() {
                            ScorePanelType::AssessmentScore(_) => {
                                let assessment = assessment_data.get();

                                if let Some(assessment) = assessment {
                                    view! {
                                        {student_info}

                                        // Assessment Details
                                        <div class="mb-6">
                                            <h3 class="text-md font-bold mb-2 flex items-center">
                                                <Icon icon=RiBookmarkBusinessLine class="w-5 h-5 mr-2" />
                                                {assessment.assessment_name}
                                            </h3>
                                            <div class="bg-[#F9F9F8] p-4 rounded-lg shadow-sm">
                                                <div class="flex justify-between mb-2">
                                                    <span class="text-gray-700">{"Subject:"}</span>
                                                    <span class="font-medium">{assessment.subject}</span>
                                                </div>
                                                <div class="flex justify-between mb-2">
                                                    <span class="text-gray-700">{"Grade Level:"}</span>
                                                    <span class="font-medium">{assessment.grade_level.unwrap_or_else(|| "Not specified".to_string())}</span>
                                                </div>
                                                <div class="flex justify-between mb-2">
                                                    <span class="text-gray-700">{"Progress:"}</span>
                                                    <span class=format!("px-2 py-1 rounded text-xs font-bold {}", get_progress_color(&assessment.progress))>
                                                        {assessment.progress.to_string()}
                                                    </span>
                                                </div>
                                                <div class="flex justify-between mb-2">
                                                    <span class="text-gray-700">{"Overall Rating:"}</span>
                                                    <span class="font-bold text-indigo-600">{assessment.assessment_rating}</span>
                                                </div>
                                            </div>
                                        </div>

                                        // Score Section
                                        <div class="mb-6">
                                            <h4 class="text-md font-bold mb-2">{"Score Progress"}</h4>
                                            <div class="bg-[#F9F9F8] p-4 rounded-lg shadow-sm">
                                                <div class="flex justify-between mb-1">
                                                    <span class="font-medium">{assessment.current_score}</span>
                                                    <span class="text-gray-500">{"out of "} {assessment.total_possible.unwrap_or(0)}</span>
                                                </div>
                                                <div class="w-full bg-gray-200 rounded-full h-2.5">
                                                    <div
                                                        class="bg-blue-600 h-2.5 rounded-full"
                                                        style=format!("width: {}%",
                                                            calculate_percentage(
                                                                assessment.current_score,
                                                                assessment.total_possible.unwrap_or(0)
                                                            )
                                                        )
                                                    ></div>
                                                </div>
                                            </div>
                                        </div>

                                        // Test Breakdown
                                        <div class="mb-6">
                                            <h4 class="text-md font-bold mb-2">{"Test Breakdown"}</h4>
                                            <div class="bg-[#F9F9F8] p-4 rounded-lg shadow-sm">
                                                <table class="min-w-full">
                                                    <thead>
                                                        <tr>
                                                            <th class="text-left text-xs font-medium text-gray-500 uppercase tracking-wider pb-2">{"Test"}</th>
                                                            <th class="text-right text-xs font-medium text-gray-500 uppercase tracking-wider pb-2">{"Score"}</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {assessment.test_details.iter().map(|test| {
                                                            view! {
                                                                <tr class="border-t border-gray-200">
                                                                    <td class="py-2">
                                                                        <a
                                                                            href="#"
                                                                            class="text-indigo-600 hover:text-indigo-800 font-medium"
                                                                        >
                                                                            {&test.test_name}
                                                                        </a>
                                                                    </td>
                                                                    <td class="py-2 text-right">
                                                                        {format!("{}/{}", test.score, test.total_possible)}
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>

                                        // Next Steps
                                        <div class="mb-6">
                                            <h4 class="text-md font-bold mb-2">{"Next Steps"}</h4>
                                            {move || {
                                                if assessment.progress != Progress::Completed {
                                                    if let Some(next_test_id) = next_test.get() {
                                                        let next_test_name = assessment.test_details.iter()
                                                            .find(|t| t.test_id == next_test_id)
                                                            .map(|t| t.test_name.clone())
                                                            .unwrap_or_else(|| "Next Test".to_string());

                                                        view! {
                                                            <div class="bg-[#F9F9F8] p-4 rounded-lg shadow-sm">
                                                                <p class="mb-4 text-sm text-gray-700">
                                                                    {"Continue this assessment by taking the next test in the sequence:"}
                                                                </p>

                                                                <GenericTestModal test_id=next_test_id test_name=next_test_name.clone()>
                                                                    <div class="w-full bg-blue-600 text-white px-4 py-2 rounded flex items-center justify-center font-medium hover:bg-blue-700">
                                                                        <span>{next_test_name}</span>
                                                                        <Icon icon=RiArrowRightSArrowsLine class="w-5 h-5 ml-2" />
                                                                    </div>
                                                                </GenericTestModal>
                                                            </div>
                                                        }.into_view()
                                                    } else {
                                                        view! {
                                                            <div class="bg-gray-100 p-4 rounded-lg text-gray-700 text-sm">
                                                                {"No additional tests are available at this time."}
                                                            </div>
                                                        }.into_view()
                                                    }
                                                } else {
                                                    view! {
                                                        <div class="bg-green-50 p-4 rounded-lg text-green-700 text-sm border border-green-200">
                                                            {"All tests in this assessment have been completed."}
                                                        </div>
                                                    }.into_view()
                                                }
                                            }}
                                        </div>

                                        // Action Buttons - Use de-anonymized student ID for navigation
                                        <div class="flex space-x-2">
                                            <A
                                                href=format!("/studentview/{}/results",
                                                    student.get().unwrap().student_id
                                                )
                                                class="flex-1 bg-indigo-600 text-white px-4 py-2 rounded text-center font-medium hover:bg-indigo-700"
                                            >
                                                {"View Full Report"}
                                            </A>
                                            <button
                                                class="flex-1 bg-gray-200 text-gray-800 px-4 py-2 rounded font-medium hover:bg-gray-300"
                                                on:click=close_panel
                                            >
                                                {"Close"}
                                            </button>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <div class="text-center py-8">
                                            <p class="text-gray-500">{"Assessment data not available"}</p>
                                        </div>
                                    }.into_view()
                                }
                            },
                            ScorePanelType::TestScore(_, _, _) => {
                                let test = test_data.get();

                                if let Some(test) = test {
                                    view! {
                                        {student_info}

                                        // Test Details
                                        <div class="mb-6">
                                            <h3 class="text-md font-bold mb-2 flex items-center">
                                                <Icon icon=RiFilePaper2DocumentLine class="w-5 h-5 mr-2" />
                                                {test.test_name}
                                            </h3>
                                            <div class="bg-[#F9F9F8] p-4 rounded-lg shadow-sm">
                                                <div class="flex justify-between mb-2">
                                                    <span class="text-gray-700">{"Test Area:"}</span>
                                                    <span class="font-medium">{test.test_area}</span>
                                                </div>
                                                <div class="flex justify-between mb-2">
                                                    <span class="text-gray-700">{"Date Administered:"}</span>
                                                    <span class="font-medium">{format_date(test.date_administered)}</span>
                                                </div>
                                                <div class="flex justify-between mb-2">
                                                    <span class="text-gray-700">{"Performance Rating:"}</span>
                                                    <span class="font-bold text-indigo-600">{test.performance_class}</span>
                                                </div>
                                            </div>
                                        </div>

                                        // Score Visual
                                        <div class="mb-6">
                                            <h4 class="text-md font-bold mb-2">{"Score"}</h4>
                                            <div class="bg-[#F9F9F8] p-4 rounded-lg shadow-sm text-center">
                                                <div class="inline-flex items-center justify-center w-32 h-32 rounded-full bg-indigo-100 mb-4">
                                                    <div class="text-center">
                                                        <div class="text-3xl font-bold text-indigo-600">{test.score}</div>
                                                        <div class="text-xs text-gray-500">{"out of"}</div>
                                                        <div class="text-lg font-medium">{test.total_possible}</div>
                                                    </div>
                                                </div>

                                                <div class="w-full bg-gray-200 rounded-full h-2.5 mb-4">
                                                    <div
                                                        class="bg-indigo-600 h-2.5 rounded-full"
                                                        style=format!("width: {}%",
                                                            calculate_percentage(test.score, test.total_possible)
                                                        )
                                                    ></div>
                                                </div>

                                                <div class="text-sm text-gray-700">
                                                    {format!("{}% Score", calculate_percentage(test.score, test.total_possible))}
                                                </div>
                                            </div>
                                        </div>

                                        // Action Buttons - Use appropriate student ID for navigation
                                        <div class="flex flex-col space-y-2">
                                            <A
                                                href=format!("/reviewtest/{}/{}/{}/{}",
                                                    test.test_id,
                                                    // Use the actual student ID (not de-anonymized) for internal navigation
                                                    student.get().map(|s| s.student_id.to_string()).unwrap_or_default(),
                                                    test.test_variant,
                                                    test.attempt,
                                                )
                                                class="w-full bg-indigo-600 text-white px-4 py-2 rounded text-center font-medium hover:bg-indigo-700"
                                            >
                                                {"Review Test Responses"}
                                            </A>
                                            <A
                                                href=format!("/studentview/{}/results",
                                                    // Use de-anonymized ID for student view navigation
                                                    student.get().map(|s| get_student_display_id(&s)).unwrap_or_default()
                                                )
                                                class="w-full bg-blue-600 text-white px-4 py-2 rounded text-center font-medium hover:bg-blue-700"
                                            >
                                                {"View Student"}
                                            </A>
                                            <button
                                                class="w-full bg-gray-200 text-gray-800 px-4 py-2 rounded font-medium hover:bg-gray-300"
                                                on:click=close_panel
                                            >
                                                {"Close"}
                                            </button>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <div class="text-center py-8">
                                            <p class="text-gray-500">{"Test data not available"}</p>
                                        </div>
                                    }.into_view()
                                }
                            },
                            ScorePanelType::None => {
                                view! {
                                    <div class="text-center py-8">
                                        <p class="text-gray-500">{"Select a score to view details"}</p>
                                    </div>
                                }.into_view()
                            }
                        }
                    }
                }}
            </div>

            <div class="p-4 mb-2 flex-shrink-0">
                {move || {
                    match panel_type.get() {
                        ScorePanelType::AssessmentScore(_) => {
                            let assessment = assessment_data.get();

                            if let Some(assessment) = assessment {
                                view! {
                                    <div class="flex space-x-2">
                                        <A
                                            href=format!("/studentview/{}/results",
                                                // Use de-anonymized ID for student view navigation
                                                student.get().map(|s| get_student_display_id(&s)).unwrap_or_default()
                                            )
                                            class="flex-1 bg-indigo-600 text-white px-4 py-2 rounded text-center font-medium hover:bg-indigo-700"
                                        >
                                            {"View Student"}
                                        </A>
                                        <button
                                            class="flex-1 bg-gray-200 text-gray-800 px-4 py-2 rounded font-medium hover:bg-gray-300"
                                            on:click=close_panel
                                        >
                                            {"Close"}
                                        </button>
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <button
                                        class="w-full bg-gray-200 text-gray-800 px-4 py-2 rounded font-medium hover:bg-gray-300"
                                        on:click=close_panel
                                    >
                                        {"Close"}
                                    </button>
                                }.into_view()
                            }
                        },
                        _ => view! { <div></div> }.into_view(),
                    }
                }}
            </div>
        </div>
    }
}
