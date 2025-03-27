use crate::app::models::student::Student;
use leptos::*;
use std::rc::Rc;

// Updated consistent color scheme and styling
const THEME_PRIMARY: &str = "#00356b"; // Darkened from #00356B
const THEME_PRIMARY_LIGHT: &str = "#5D7A9E"; // Darkened from #7F9AB5
const THEME_GRAY_BG: &str = "#F0F2F5";

// Improved consistent styling with better naming
const CARD_CONTAINER: &str = "h-full bg-white p-6 border-t-4 border-l border-r border-b border-gray-200 shadow-md rounded-lg flex flex-col";
const SECTION_CONTAINER: &str = "bg-gray-50 p-5 rounded-lg border border-gray-100 shadow-sm";
const SECTION_TITLE: &str =
    "text-sm font-semibold text-gray-700 mb-3 pb-2 border-b border-gray-200";
const INFO_TITLE: &str = "text-xs text-gray-600 font-medium"; // Darkened from text-gray-500
const INFO_VALUE: &str = "text-gray-900 mt-1";const INFO_GROUP: &str = "mb-4";
const BUTTON_CONTAINER: &str =
    "mt-6 pt-4 flex gap-3 justify-end sticky bottom-0 bg-white border-t border-gray-200";
const BUTTON_PRIMARY: &str = 
    "px-4 py-2 bg-[#00356b] rounded-md font-medium text-white hover:font-large transition-colors hover:bg-[#00457b] hover:border-white";
const BUTTON_SECONDARY: &str = 
    "px-4 py-2 bg-gray-200 rounded-md font-medium text-gray-500 hover:text-gray-900 hover:border-gray-700 transition-colors border border-gray-300";
const BUTTON_ACCENT: &str = 
    "px-4 py-2 bg-[#FCEDA0] rounded-md font-medium text-gray-500 hover:text-gray-900 transition-colors border border-gray-300 hover:border-gray-700";

#[component]
pub fn StudentDetails(
    #[prop()] student: Rc<Student>,
    #[prop(optional)] on_edit_student: Option<Callback<()>>,
) -> impl IntoView {
    // Create a memo for the student to ensure stable references
    let student_memo = create_memo(move |_| student.clone());

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
                                    <div class="h-4 w-4 rounded-full bg-green-600 mr-2"></div>
                                    <span class="text-green-700 font-medium">"Active"</span>
                                </div>
                            }
                        } else {
                            view! { <div><span class="text-gray-500 font-medium">"Inactive"</span></div> }
                        }}
                    </div>
                </div>
            }
        };

        if student.iep {
            services.push(create_service_item("IEP Status", true));
        }

        if student.bip {
            services.push(create_service_item("BIP Status", true));
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

        if student.intervention {
            services.push(create_service_item("Intervention", true));
        }

        if student.eye_glasses {
            services.push(create_service_item("Glasses", true));
        }

        if student.esl.to_string() != "Not Applicable" {
            services.push(view! {
                <div class=INFO_GROUP>
                    <div class=INFO_TITLE>"ESL Status"</div>
                    <div class=INFO_VALUE>
                        <span class="px-2 py-1 bg-blue-100 text-blue-800 rounded-md text-xs font-medium">
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
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-xl font-bold text-gray-800">
                    {move || format!("{} {}", student_memo().firstname, student_memo().lastname)}
                </h2>
                <div class="px-3 py-1 rounded-full bg-blue-200 text-blue-800 text-xs font-medium">
                    {move || student_memo().grade.to_string()}
                </div>
            </div>

            <div class="flex-grow overflow-y-auto space-y-6">
                // Basic Information Section
                <div>
                    <h3 class=SECTION_TITLE>"Basic Information"</h3>
                    <div class=SECTION_CONTAINER>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div class=INFO_GROUP>
                                <div class=INFO_TITLE>"Preferred Name"</div>
                                <div class=INFO_VALUE>{move || student_memo().preferred.clone()}</div>
                            </div>

                            <div class=INFO_GROUP>
                                <div class=INFO_TITLE>"Student ID"</div>
                                <div class=INFO_VALUE>{move || format!("#{}", student_memo().student_id)}</div>
                            </div>

                            <div class=INFO_GROUP>
                                <div class=INFO_TITLE>"Teacher"</div>
                                <div class=INFO_VALUE>{move || student_memo().teacher.clone()}</div>
                            </div>

                            <div class=INFO_GROUP>
                                <div class=INFO_TITLE>"Date of Birth"</div>
                                <div class=INFO_VALUE>{move || format!("{}", student_memo().date_of_birth.format("%m-%d-%Y"))}</div>
                            </div>
                        </div>
                    </div>
                </div>

                // Support Services Section
                <div>
                    <h3 class=SECTION_TITLE>"Support Services"</h3>
                    <div class=SECTION_CONTAINER>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
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
                            <div class="mt-2 whitespace-pre-wrap text-gray-700 bg-white p-3 rounded border border-gray-200 min-h-12">
                                {move || {
                                    let notes = student_memo().notes.clone();
                                    if notes.is_empty() {
                                        view! { <span class="text-gray-400 italic">"No notes available"</span> }
                                    } else {
                                        view! { <span>{notes}</span> }
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // Button container at the bottom
            <div class=BUTTON_CONTAINER>
                <button class=BUTTON_SECONDARY>
                    "Next Student"
                </button>
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
                    "Test Results"
                </button>
            </div>
        </div>
    }
}
