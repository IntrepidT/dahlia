use crate::app::models::employee::EmployeeRole;
use leptos::prelude::*;
use leptos::prelude::*;
use strum::IntoEnumIterator;

const SEARCH_CONTAINER_STYLE: &str = "flex flex-wrap gap-4 items-end mb-8 mt-20";
const INPUT_STYLE: &str = "w-full px-4 py-2 bg-white border border-gray-200 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-gray-200 transition-all duration-200";
const LABEL_STYLE: &str = "block text-sm font-medium text-gray-700 mb-1";
const BUTTON_STYLE: &str = "px-4 py-2 text-sm font-medium text-gray-600 bg-white border border-[#DADADA] rounded-md shadow-sm hover:bg-gray-50 transition-all duration-200";

#[component]
pub fn SearchFilter(
    #[prop(into)] search_term: Signal<String>,
    #[prop(into)] set_search_term: WriteSignal<String>,
    #[prop(into)] role_filter: Signal<String>,
    #[prop(into)] set_role_filter: WriteSignal<String>,
    #[prop(into)] on_clear_filters: Callback<()>,
) -> impl IntoView {
    view! {
        <div class=SEARCH_CONTAINER_STYLE>
            <div class="flex-1 min-w-64">
                <label for="search" class=LABEL_STYLE>"Search Employees"</label>
                <div class="relative">
                    <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none text-gray-400">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
                        </svg>
                    </div>
                    <input
                        type="text"
                        name="search"
                        id="search"
                        class=format!("{} pl-10", INPUT_STYLE)
                        placeholder="Search by name..."
                        prop:value={search_term}
                        on:input=move |ev| set_search_term(event_target_value(&ev))
                    />
                </div>
            </div>

            <div class="w-64">
                <label for="role-filter" class=LABEL_STYLE>"Filter by Role"</label>
                <select
                    id="role-filter"
                    class=INPUT_STYLE
                    prop:value={role_filter}
                    on:change=move |ev| set_role_filter(event_target_value(&ev))
                >
                    <option value="">"All Roles"</option>
                    {EmployeeRole::iter().map(|role| view! {
                        <option value=format!("{}", role)>
                            {format!("{}", role)}
                        </option>
                    }).collect::<Vec<_>>()}
                </select>
            </div>

            <div>
                <button
                    type="button"
                    class=BUTTON_STYLE
                    on:click=move |_| on_clear_filters.run(())
                >
                    "Clear Filters"
                </button>
            </div>
        </div>
    }
}
