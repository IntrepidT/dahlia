use leptos::*;

const SEARCH_CONTAINER_STYLE: &str = "mt-10 mb-4 flex flex-wrap gap-4 items-center";
const INPUT_STYLE: &str = "focus:ring-indigo-500 focus:border-indigo-500 block w-full pl-3 pr-3 sm:text-sm border-gray-300 rounded-md h-10 border";
const SELECT_STYLE: &str = "mt-1 block w-full pl-3 pr-10 py-2 text-base bg-white shadow-sm border-gray-200 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md h-10 border transition-all";
const CHECKBOX_STYLE: &str =
    "form-checkbox mr-2 h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded";
const LABEL_STYLE: &str = "block text-sm font-medium text-gray-700 mb-1";
const CHECKBOX_CONTAINER_STYLE: &str = "flex items-center mt-6";

#[derive(Clone)]
pub struct FilterState {
    pub search_term: String,
    pub grade_filter: String,
    pub iep_filter: bool,
    pub esl_filter: bool,
    pub teacher_filter: String,
}

#[component]
pub fn SearchFilter(
    #[prop(into)] set_search_term: Callback<String>,
    #[prop(into)] set_grade_filter: Callback<String>,
    #[prop(into)] set_teacher_filter: Callback<String>,
    #[prop(into)] set_iep_filter: Callback<bool>,
    #[prop(into)] set_esl_filter: Callback<bool>,
    #[prop(into)] set_bip_filter: Callback<bool>,
    #[prop(into)] teachers: Signal<Vec<String>>,
    #[prop(into)] search_term: Signal<String>,
    #[prop(into)] on_clear_filters: Callback<()>,
) -> impl IntoView {
    let iep_checkbox_ref = create_node_ref::<html::Input>();
    let esl_checkbox_ref = create_node_ref::<html::Input>();
    let bip_checkbox_ref = create_node_ref::<html::Input>();
    let grade_filter_ref = create_node_ref::<html::Select>();
    let teacher_filter_ref = create_node_ref::<html::Select>();
    view! {
        <div class=SEARCH_CONTAINER_STYLE>
            // Search input
            <div class="flex-1">
                <label for="search" class=LABEL_STYLE>"Search Students"</label>
                <div class="relative rounded-md shadow-sm">
                    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                        <span class="text-gray-500 sm:text-sm">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
                            </svg>
                        </span>
                    </div>
                    <input
                        type="text"
                        name="search"
                        id="search"
                        class="focus:ring-indigo-500 focus:border-indigo-500 block w-full pl-10 pr-12 sm:text-sm border-gray-300 rounded-md h-10 border"
                        placeholder="Search students..."
                        prop:value={move || search_term.get()}
                        on:input=move |ev| set_search_term(event_target_value(&ev))
                    />
                </div>
            </div>

            // Grade filter dropdown
            <div class="w-48">
                <label for="grade-filter" class=LABEL_STYLE>"Filter by Grade"</label>
                <select
                    id="grade-filter"
                    class=SELECT_STYLE
                    on:change=move |ev| set_grade_filter(event_target_value(&ev))
                    node_ref=grade_filter_ref
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
            </div>

            // Teacher filter dropdown
            <div class="w-48">
                <label for="teacher-filter" class=LABEL_STYLE>"Filter by Teacher"</label>
                <select
                    id="teacher-filter"
                    class=SELECT_STYLE
                    on:change=move |ev| set_teacher_filter(event_target_value(&ev))
                    node_ref=teacher_filter_ref
                >
                    <option value="all">"All Teachers"</option>
                    {move || {
                        let teacher_list = teachers.get();
                        log::info!("Rendering teacher dropdown with {} teachers", teacher_list.len());

                        teacher_list.into_iter().map(|teacher| {
                            view! {
                                <option value={teacher.clone()}>{teacher}</option>
                            }
                        }).collect_view()
                    }}
                </select>
            </div>

            // Checkboxes container
            <div class="flex flex-wrap gap-6">
                // IEP filter checkbox
                <div class=CHECKBOX_CONTAINER_STYLE>
                    <input
                        type="checkbox"
                        id="iep-filter"
                        class=CHECKBOX_STYLE
                        on:change=move |ev| set_iep_filter(event_target_checked(&ev))
                        node_ref=iep_checkbox_ref
                    />
                    <label for="iep-filter" class="text-sm text-gray-700">"IEP Students"</label>
                </div>

                // ESL filter checkbox
                <div class=CHECKBOX_CONTAINER_STYLE>
                    <input
                        type="checkbox"
                        id="esl-filter"
                        class=CHECKBOX_STYLE
                        on:change=move |ev| set_esl_filter(event_target_checked(&ev))
                        node_ref=esl_checkbox_ref
                    />
                    <label for="esl-filter" class="text-sm text-gray-700">"ESL Students"</label>
                </div>

                // BIP filter checkbox
                <div class=CHECKBOX_CONTAINER_STYLE>
                    <input
                        type="checkbox"
                        id="bip-filter"
                        class=CHECKBOX_STYLE
                        on:change=move |ev| set_bip_filter(event_target_checked(&ev))
                        node_ref=bip_checkbox_ref
                    />
                    <label for="bip-filter" class="text-sm text-gray-700">"BIP Students"</label>
                </div>
            </div>

            // Clear filters button
            <div class="flex items-end mt-6">
                <button
                    type="button"
                    class="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 h-10"
                    on:click=move |_| {
                        if let Some(input) = iep_checkbox_ref.get() {
                            input.set_checked(false);
                        }
                        if let Some(input) = esl_checkbox_ref.get() {
                            input.set_checked(false);
                        }
                        if let Some(input) = bip_checkbox_ref.get() {
                            input.set_checked(false);
                        }
                        if let Some(select) = grade_filter_ref.get() {
                            select.set_value("all");
                        }
                        if let Some(select) = teacher_filter_ref.get() {
                            select.set_value("all");
                        }

                        on_clear_filters.call(());
                    }
                >
                    "Clear Filters"
                </button>
            </div>
        </div>
    }
}
