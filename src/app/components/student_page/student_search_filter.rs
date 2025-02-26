use crate::app::models::student::GradeEnum;
use leptos::*;

// Styles
const CHECKBOX_CONTAINER_STYLE: &str = "flex items-center gap-2 bg-white rounded-lg px-4 py-3";

#[derive(Clone)]
pub struct FilterState {
    pub search_term: String,
    pub grade_filter: String,
    pub iep_filter: bool,
    pub ell_filter: bool,
    pub teacher_filter: String,
}

#[component]
pub fn SearchFilter(
    // Props
    #[prop(into)] set_search_term: Callback<String>,
    #[prop(into)] set_grade_filter: Callback<String>,
    #[prop(into)] set_teacher_filter: Callback<String>,
    #[prop(into)] set_iep_filter: Callback<bool>,
    #[prop(into)] set_ell_filter: Callback<bool>,
    #[prop(optional)] teachers: Option<Vec<String>>,
) -> impl IntoView {
    view! {
        <div class="bg-[#00356b] rounded-lg p-6 mb-6">
            <div class="flex gap-4 flex-wrap">
                // Search input
                <div class="relative flex-grow max-w-[20rem]">
                    <input
                        type="text"
                        placeholder="Search students..."
                        class="w-full p-3 pl-4 rounded-lg"
                        on:input=move |ev| {
                            set_search_term(event_target_value(&ev));
                        }
                    />
                </div>

                // Grade filter dropdown
                <select
                    class="p-3 rounded-lg"
                    on:change=move |ev| set_grade_filter(event_target_value(&ev))
                >
                    <option value="all">"All Grades"</option>
                    <option value="Kindergarten">"K"</option>
                    <option value="1st Grade">"1st"</option>
                    <option value="2nd Grade">"2nd"</option>
                    <option value="3rd Grade">"3rd"</option>
                    <option value="4th Grade">"4th"</option>
                    <option value="5th Grade">"5th"</option>
                    <option value="6th Grade">"6th"</option>
                    <option value="7th Grade">"7th"</option>
                    <option value="8th Grade">"8th"</option>
                    <option value="9th Grade">"9th"</option>
                    <option value="10th Grade">"10th"</option>
                    <option value="11th Grade">"11th"</option>
                    <option value="12th Grade">"12th"</option>
                </select>

                // Teacher filter dropdown
                <select
                    class="p-3 rounded-lg"
                    on:change=move |ev| set_teacher_filter(event_target_value(&ev))
                >
                    <option value="all">"Teacher"</option>
                    {move || {
                        teachers.clone().unwrap_or_default().into_iter().map(|teacher| {
                            view! {
                                <option value={teacher.clone()}>{teacher}</option>
                            }
                        }).collect_view()
                    }}
                </select>

                // IEP filter checkbox
                <div class=CHECKBOX_CONTAINER_STYLE>
                    <input
                        type="checkbox"
                        id="iep-filter"
                        class="form-checkbox h-5 w-5 text-[#00356b]"
                        on:change=move |ev| set_iep_filter(event_target_checked(&ev))
                    />
                    <label for="iep-filter">"Show IEP Students"</label>
                </div>

                // ELL filter checkbox
                <div class=CHECKBOX_CONTAINER_STYLE>
                    <input
                        type="checkbox"
                        id="ell-filter"
                        class="form-checkbox h-5 w-5 text-[#00356b]"
                        on:change=move |ev| set_ell_filter(event_target_checked(&ev))
                    />
                    <label for="ell-filter">"Show ELL Students"</label>
                </div>
            </div>
        </div>
    }
}
