use crate::app::models::student::Student;
use crate::app::middleware::global_settings::use_settings;
use crate::app::components::auth::enhanced_login_form::{use_student_mapping_service, DeAnonymizedStudent};
use leptos::*;
use leptos_router::*;
use std::rc::Rc;

// Updated color scheme to match the palette
const THEME_PRIMARY: &str = "#2E3A59"; // Navy blue
const THEME_SECONDARY: &str = "#DADADA"; // Light gray
const THEME_BG: &str = "#F9F9F8"; // Off-white

// Improved consistent styling with better naming and responsive design
const CARD_CONTAINER: &str = "h-[95%] bg-[#F9F9F8] p-3 sm:p-6 border-t-8 border-[#2E3A59] shadow-md rounded-lg flex flex-col";
const SECTION_CONTAINER: &str = "bg-white p-3 sm:p-5 rounded-lg border border-[#DADADA] shadow-sm";
const SECTION_TITLE: &str =
    "text-xs sm:text-sm font-semibold text-[#2E3A59] mb-2 sm:mb-3 pb-2 border-b border-[#DADADA]";
const INFO_TITLE: &str = "text-xs text-[#2E3A59] text-opacity-70 font-medium";
const INFO_VALUE: &str = "text-xs sm:text-sm text-[#2E3A59] mt-1";
const INFO_GROUP: &str = "mb-3 sm:mb-4";
const BUTTON_CONTAINER: &str =
    "mt-4 sm:mt-6 pt-3 sm:pt-4 flex flex-wrap sm:flex-nowrap gap-2 sm:gap-3 justify-end sticky bottom-0 bg-[#F9F9F8] border-t border-[#DADADA]";
const BUTTON_PRIMARY: &str = 
    "w-full sm:w-auto px-3 sm:px-4 py-2 bg-[#2E3A59] rounded-md text-xs sm:text-sm font-medium text-[#F9F9F8] hover:bg-opacity-80 transition-colors";
const BUTTON_SECONDARY: &str = 
    "w-full sm:w-auto px-3 sm:px-4 py-2 bg-[#F9F9F8] rounded-md text-xs sm:text-sm font-medium text-[#2E3A59] hover:bg-opacity-80 transition-colors border border-[#DADADA]";
const BUTTON_ACCENT: &str = 
    "w-full sm:w-auto px-3 sm:px-4 py-2 bg-[#F9F9F8] rounded-md text-xs sm:text-sm font-medium text-[#2E3A59] hover:bg-[#DADADA] hover:bg-opacity-30 transition-colors border border-[#DADADA]";

#[component]
pub fn StudentDetails(
    #[prop()] student: Rc<Student>,
    #[prop(optional)] on_edit_student: Option<Callback<()>>,
) -> impl IntoView {
    // Get global settings
    let (settings, _) = use_settings();
    let anonymization_enabled = move || settings.get().student_protections;
    
    // Get the mapping service
    let (student_mapping_service, _) = use_student_mapping_service();
    
    // Clone the student Rc for use in closures
    let student_for_deanonymization = student.clone();
    let student_for_memo = student.clone();
    
    // Create a memo for the de-anonymized student info
    let de_anonymized_student = create_memo(move |_| {
        let mapping_service = student_mapping_service.get();
        DeAnonymizedStudent::from_student_with_mapping(
            &student_for_deanonymization.as_ref(), 
            mapping_service.as_ref()
        )
    });
    
    // Create a memo for the student to ensure stable references
    let student_memo = create_memo(move |_| student_for_memo.clone());

    // Function to get display name based on anonymization settings
    let get_display_name = move || {
        if anonymization_enabled() {
            de_anonymized_student.get().display_name
        } else {
            format!(
                "{} {}", 
                student_memo().firstname.as_ref().unwrap_or(&"Unknown".to_string()), 
                student_memo().lastname.as_ref().unwrap_or(&"Unknown".to_string())
            )
        }
    };

    // Function to get display ID based on anonymization settings
    let get_display_id = move || {
        if anonymization_enabled() {
            de_anonymized_student.get().display_id
        } else {
            student_memo().student_id.to_string()
        }
    };

    // Function to get student PIN with proper de-anonymization
    let get_student_pin = move || {
        if anonymization_enabled() {
            if let Some(mapping_service) = student_mapping_service.get() {
                if let Some(mapping) = mapping_service.get_original_student_info(student_memo().student_id) {
                    return mapping.pin.clone();
                }
            }
            // Fallback to anonymized display
            "****".to_string()
        } else {
            student_memo().pin.unwrap_or(0).to_string()
        }
    };

    // Function to create support services view that doesn't borrow student directly
    let support_services_view = move || {
        let student = student_memo();
        let mut services = Vec::new();

        // Function to create consistent service status item
        let create_service_item = |title: &'static str, active: bool| {
            view! {
                <div class=INFO_GROUP>
                    <div class=INFO_TITLE>{title}</div>
                    <div class=INFO_VALUE>
                        {if active {
                            view! {
                                <div class="flex items-center">
                                    <div class="h-3 w-3 sm:h-4 sm:w-4 rounded-full bg-green-600 mr-1 sm:mr-2"></div>
                                    <span class="text-xs sm:text-sm text-green-700 font-medium">"Active"</span>
                                </div>
                            }
                        } else {
                            view! { <div><span class="text-xs sm:text-sm text-[#2E3A59] text-opacity-50 font-medium">"Inactive"</span></div> }
                        }}
                    </div>
                </div>
            }
        };

        if student.iep {
            services.push(create_service_item("IEP Status", true));
        }

        if student.bip {
            services.push(create_service_item("BEH Status", true));
        }

        if student.student_504 {
            services.push(create_service_item("504 Status", true));
        }

        if student.gt {
            services.push(create_service_item("GT Status", true));
        }

        if student.readplan {
            services.push(create_service_item("Readplan", true));
        }

        if student.intervention.is_some() {
            services.push(view! {
                <div class=INFO_GROUP>
                    <div class=INFO_TITLE>"Intervention Status"</div>
                    <div class=INFO_VALUE>
                        {match &student.intervention {
                            Some(intervention) => {
                                view! {
                                    <span class="px-1 sm:px-2 py-0.5 sm:py-1 bg-[#2E3A59] bg-opacity-10 text-[#2E3A59] rounded-md text-xs font-medium">
                                        {intervention.to_string()}
                                    </span>
                                }
                            },
                            None => {
                                view! {
                                    <span class="text-xs sm:text-sm text-[#2E3A59] text-opacity-50 font-medium">"None"</span>
                                }
                            }
                        }}
                </div>
                </div>
            });
        }

        if student.eye_glasses {
            services.push(create_service_item("Glasses", true));
        }

        if student.esl.to_string() != "Not Applicable" {
            services.push(view! {
                <div class=INFO_GROUP>
                    <div class=INFO_TITLE>"ESL Status"</div>
                    <div class=INFO_VALUE>
                        <span class="px-1 sm:px-2 py-0.5 sm:py-1 bg-[#2E3A59] bg-opacity-10 text-[#2E3A59] rounded-md text-xs font-medium">
                            {student.esl.to_string()}
                        </span>
                    </div>
                </div>
            });
        }

        services
    };

    view! {
        <div class=CARD_CONTAINER>
            <div class="flex items-center justify-between mb-3 sm:mb-6">
                <h2 class="text-lg sm:text-xl font-bold text-[#2E3A59]">
                    {get_display_name}
                </h2>
                <div class="px-2 sm:px-3 py-0.5 sm:py-1 rounded-full bg-[#2E3A59] text-white text-xs font-medium">
                    {move || student_memo().current_grade_level.to_string()}
                </div>
            </div>

            // Show anonymization status indicator
            {move || {
                if anonymization_enabled() {
                    let has_mapping = student_mapping_service.get()
                        .map(|service| service.has_mapping_for_app_id(student_memo().student_id))
                        .unwrap_or(false);
                    
                    view! {
                        <div class="mb-4 p-2 rounded-md border text-xs">
                            {if has_mapping {
                                view! {
                                    <div class="flex items-center text-green-700 bg-green-50 border-green-200">
                                        <div class="h-2 w-2 rounded-full bg-green-600 mr-2"></div>
                                        "Student identity available - showing real information"
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="flex items-center text-amber-700 bg-amber-50 border-amber-200">
                                        <div class="h-2 w-2 rounded-full bg-amber-600 mr-2"></div>
                                        "Student identity protected - showing anonymized information"
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            <div class="flex-grow overflow-y-auto space-y-4 sm:space-y-6">
                // Basic Information Section
                <div>
                    <h3 class=SECTION_TITLE>"Basic Information"</h3>
                    <div class=SECTION_CONTAINER>
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-6">
                            <div class=INFO_GROUP>
                                <div class=INFO_TITLE>"Preferred Name"</div>
                                <div class=INFO_VALUE>{move || student_memo().preferred.clone()}</div>
                            </div>

                            <div class=INFO_GROUP>
                                <div class=INFO_TITLE>
                                    {move || if anonymization_enabled() { "Student ID" } else { "Student ID" }}
                                </div>
                                <div class=INFO_VALUE>
                                    {move || format!("#{}", get_display_id())}
                                </div>
                            </div>

                            <div class=INFO_GROUP>
                                <div class=INFO_TITLE>"Teacher"</div>
                                <div class=INFO_VALUE>{move || student_memo().teacher.clone()}</div>
                            </div>

                            <div class=INFO_GROUP>
                                <div class=INFO_TITLE>"Date of Birth"</div>
                                <div class=INFO_VALUE>
                                    {move || {
                                        if anonymization_enabled() {
                                            let has_mapping = student_mapping_service.get()
                                                .map(|service| service.has_mapping_for_app_id(student_memo().student_id))
                                                .unwrap_or(false);
                                            if has_mapping {
                                                format!("{}", student_memo().date_of_birth.format("%m-%d-%Y"))
                                            } else {
                                                "Protected".to_string()
                                            }
                                        } else {
                                            format!("{}", student_memo().date_of_birth.format("%m-%d-%Y"))
                                        }
                                    }}
                                </div>
                            </div>
                            
                            <div class=INFO_GROUP>
                                <div class=INFO_TITLE>"Student Pin"</div>
                                <div class=INFO_VALUE>{get_student_pin}</div>
                            </div>
                        </div>
                    </div>
                </div>

                // Support Services Section
                <div>
                    <h3 class=SECTION_TITLE>"Support Services"</h3>
                    <div class=SECTION_CONTAINER>
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-6">
                            {support_services_view}
                        </div>
                    </div>
                </div>

                // Additional Information Section
                <div>
                    <h3 class=SECTION_TITLE>"Additional Information"</h3>
                    <div class=SECTION_CONTAINER>
                        <div class=INFO_GROUP>
                            <div class=INFO_TITLE>"Student Notes"</div>
                            <div class="mt-2 whitespace-pre-wrap text-[#2E3A59] bg-white p-2 sm:p-3 rounded border border-[#DADADA] min-h-10 sm:min-h-12 text-xs sm:text-sm">
                                {move || {
                                    let notes = student_memo().notes.clone();
                                    if notes.is_empty() {
                                        view! { <span class="text-[#2E3A59] text-opacity-40 italic">"No notes available"</span> }
                                    } else {
                                        view! { <span>{notes}</span> }
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // Button container at the bottom - stacked on mobile
            <div class=BUTTON_CONTAINER>
                <button class=BUTTON_ACCENT
                    on:click=move |_| {
                        if let Some(callback) = on_edit_student {
                            callback.call(());
                        }
                    }
                >
                    "Edit Student"
                </button>
                <button class=BUTTON_PRIMARY>
                    <a href=format!("/studentview/{}/results", &student_memo().student_id)>
                        "Test Results"
                    </a>
                </button>
            </div>
        </div>
    }
}
