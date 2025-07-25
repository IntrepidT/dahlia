use leptos::*;

// More responsive container style with padding adjustments for small screens
const SEARCH_CONTAINER_STYLE: &str =
    "md:mt-16 mt-14 mb-4 flex flex-grow gap-2 items-center w-full justify-between";
// Improved input style with better handling for small screens
const INPUT_STYLE: &str = "focus:ring-indigo-500 focus:border-indigo-500 block w-full pl-3 pr-3 text-xs sm:text-sm border-gray-300 rounded-md h-8 sm:h-10 border";
// Responsive select style
const SELECT_STYLE: &str = "mt-1 block w-full pl-2 pr-6 sm:pl-3 sm:pr-10 py-1 sm:py-2 text-xs sm:text-sm bg-white shadow-sm border-gray-200 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 rounded-md h-8 sm:h-10 border transition-all";
const CHECKBOX_STYLE: &str =
    "form-checkbox mr-1 sm:mr-2 h-3 w-3 sm:h-4 sm:w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded";
const LABEL_STYLE: &str = "block text-xs sm:text-sm font-medium text-gray-700 mb-1";
const CHECKBOX_CONTAINER_STYLE: &str = "flex items-center mt-3 sm:mt-6";

// Extra filters container for expanded view
const EXPANDED_FILTERS_STYLE: &str = "w-full flex flex-wrap gap-2 sm:gap-4 mt-2 pb-2";

#[derive(Clone)]
pub struct FilterState {
    pub search_term: String,
    pub grade_filter: String,
    pub iep_filter: bool,
    pub esl_filter: bool,
    pub teacher_filter: String,
    pub intervention_filter: String,
    pub student_504_filter: bool,
    pub readplan_filter: bool,
    pub gt_filter: bool,
    pub bip_filter: bool,
}

#[component]
pub fn SearchFilter(
    #[prop(into)] set_search_term: Callback<String>,
    #[prop(into)] set_grade_filter: Callback<String>,
    #[prop(into)] set_teacher_filter: Callback<String>,
    #[prop(into)] set_iep_filter: Callback<bool>,
    #[prop(into)] set_esl_filter: Callback<bool>,
    #[prop(into)] set_intervention_filter: Callback<String>,
    #[prop(into)] set_student_504_filter: Callback<bool>,
    #[prop(into)] set_readplan_filter: Callback<bool>,
    #[prop(into)] set_gt_filter: Callback<bool>,
    #[prop(into)] set_bip_filter: Callback<bool>,
    #[prop(into)] teachers: Signal<Vec<String>>,
    #[prop(into)] search_term: Signal<String>,
    #[prop(into)] on_clear_filters: Callback<()>,
    #[prop(into)] is_panel_expanded: Signal<bool>,
) -> impl IntoView {
    let iep_checkbox_ref = create_node_ref::<html::Input>();
    let esl_checkbox_ref = create_node_ref::<html::Input>();
    let student_504_checkbox_ref = create_node_ref::<html::Input>();
    let readplan_checkbox_ref = create_node_ref::<html::Input>();
    let gt_checkbox_ref = create_node_ref::<html::Input>();
    let bip_checkbox_ref = create_node_ref::<html::Input>();
    let intervention_filter_ref = create_node_ref::<html::Select>();
    let grade_filter_ref = create_node_ref::<html::Select>();
    let teacher_filter_ref = create_node_ref::<html::Select>();

    view! {
        <div class=SEARCH_CONTAINER_STYLE>
            // Search input - adjusted to be less wide
            <div class="flex-grow sm:w-72 md:w-72">
                <label for="search" class=LABEL_STYLE>"Search Students"</label>
                <div class="relative rounded-md shadow-sm">
                    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                        <span class="text-gray-500 sm:text-sm">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 sm:h-5 sm:w-5" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
                            </svg>
                        </span>
                    </div>
                    <input
                        type="text"
                        name="search"
                        id="search"
                        class="focus:ring-indigo-500 focus:border-indigo-500 block w-full pl-10 pr-10 text-xs sm:text-sm border-gray-300 rounded-md h-8 sm:h-10 border"
                        placeholder="Search students..."
                        prop:value={move || search_term.get()}
                        on:input=move |ev| set_search_term(event_target_value(&ev))
                    />
                </div>
            </div>

            // Grade filter dropdown - optimized width
            <div class="flex-grow sm:w-36 md:w-36">
                <label for="grade-filter" class=LABEL_STYLE>"Grade"</label>
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

            // Teacher filter dropdown - optimized width
            <div class="flex-grow sm:w-36 md:w-36">
                <label for="teacher-filter" class=LABEL_STYLE>"Teacher"</label>
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

            // Intervention filter - optimized width
            <div class="flex-grow sm:w-36 md:w-36">
                <label for="intervention-filter" class=LABEL_STYLE>"Intervention"</label>
                <select
                    id="intervention-filter"
                    class=SELECT_STYLE
                    on:change=move |ev| set_intervention_filter(event_target_value(&ev))
                    node_ref=intervention_filter_ref
                >
                    <option value="all">""</option>
                    <option value="Literacy">"Literacy"</option>
                    <option value="Math">"Math"</option>
                    <option value="Literacy and Math">"Literacy and Math"</option>
                    <option value="None">"Exclude Intervention"</option>
                </select>
            </div>

            // Always render all checkboxes but use CSS to control visibility
            <div class="flex flex-grow items-center gap-3 mt-4 ml-2">
                // IEP filter checkbox
                <div class="flex items-center mr-2">
                    <input
                        type="checkbox"
                        id="iep-filter"
                        class=CHECKBOX_STYLE
                        on:change=move |ev| set_iep_filter(event_target_checked(&ev))
                        node_ref=iep_checkbox_ref
                    />
                    <label for="iep-filter" class="text-xs sm:text-sm text-gray-700">"IEP"</label>
                </div>

                // ESL filter checkbox
                <div class="flex items-center mr-2">
                    <input
                        type="checkbox"
                        id="esl-filter"
                        class=CHECKBOX_STYLE
                        on:change=move |ev| set_esl_filter(event_target_checked(&ev))
                        node_ref=esl_checkbox_ref
                    />
                    <label for="esl-filter" class="text-xs sm:text-sm text-gray-700">"ESL"</label>
                </div>

                // 504 Plan filter - moved to main row
                <div class="flex items-center mr-2">
                    <input
                        type="checkbox"
                        id="504-filter"
                        class=CHECKBOX_STYLE
                        on:change=move |ev| set_student_504_filter(event_target_checked(&ev))
                        node_ref=student_504_checkbox_ref
                    />
                    <label for="504-filter" class="text-xs sm:text-sm text-gray-700">"504"</label>
                </div>

                // Always render these but use conditional styling instead of conditional rendering
                <div class=move || {
                    if is_panel_expanded.get() {
                        "hidden".to_string()
                    } else {
                        "flex items-center mr-2".to_string()
                    }
                }>
                    <input
                        type="checkbox"
                        id="readplan-filter"
                        class=CHECKBOX_STYLE
                        on:change=move |ev| set_readplan_filter(event_target_checked(&ev))
                        node_ref=readplan_checkbox_ref
                    />
                    <label for="readplan-filter" class="text-xs sm:text-sm text-gray-700">"Read Plan"</label>
                </div>

                <div class=move || {
                    if is_panel_expanded.get() {
                        "hidden".to_string()
                    } else {
                        "flex items-center mr-2".to_string()
                    }
                }>
                    <input
                        type="checkbox"
                        id="gt-filter"
                        class=CHECKBOX_STYLE
                        on:change=move |ev| set_gt_filter(event_target_checked(&ev))
                        node_ref=gt_checkbox_ref
                    />
                    <label for="gt-filter" class="text-xs sm:text-sm text-gray-700">"GT"</label>
                </div>

                <div class=move || {
                    if is_panel_expanded.get() {
                        "hidden".to_string()
                    } else {
                        "flex items-center mr-2".to_string()
                    }
                }>
                    <input
                        type="checkbox"
                        id="bip-filter"
                        class=CHECKBOX_STYLE
                        on:change=move |ev| set_bip_filter(event_target_checked(&ev))
                        node_ref=bip_checkbox_ref
                    />
                    <label for="bip-filter" class="text-xs sm:text-sm text-gray-700">"BEH"</label>
                </div>
            </div>

            // Clear filters button
            <div class="flex items-center mt-3 flex-shrink-0">
                <button
                    type="button"
                    class="inline-flex justify-center items-center px-3 sm:px-4 py-1 sm:py-2 border border-gray-300 rounded-md shadow-sm text-xs sm:text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 h-8 sm:h-10 transition-none whitespace-nowrap"
                    on:click=move |_| {
                        if let Some(input) = iep_checkbox_ref.get() {
                            input.set_checked(false);
                        }
                        if let Some(input) = esl_checkbox_ref.get() {
                            input.set_checked(false);
                        }
                        if let Some(input) = intervention_filter_ref.get() {
                            input.set_value("all");
                        }
                        if let Some(select) = grade_filter_ref.get() {
                            select.set_value("all");
                        }
                        if let Some(select) = teacher_filter_ref.get() {
                            select.set_value("all");
                        }
                        // Clear the new checkbox filters too
                        if let Some(input) = student_504_checkbox_ref.get() {
                            input.set_checked(false);
                        }
                        if let Some(input) = readplan_checkbox_ref.get() {
                            input.set_checked(false);
                        }
                        if let Some(input) = gt_checkbox_ref.get() {
                            input.set_checked(false);
                        }
                        if let Some(input) = bip_checkbox_ref.get() {
                            input.set_checked(false);
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
