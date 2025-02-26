use crate::app::models::employee::EmployeeRole;
use leptos::*;
use strum::IntoEnumIterator;

const SEARCH_CONTAINER_STYLE: &str = "mb-4 flex gap-4 items-center mt-10";

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
            <div class="flex-1">
                <label for="search" class="block text-sm font-medium text-gray-700 mb-1">"Search Employees"</label>
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
                        placeholder="Search by name..."
                        prop:value={search_term}
                        on:input=move |ev| set_search_term(event_target_value(&ev))
                    />
                </div>
            </div>

            <div class="w-64">
                <label for="role-filter" class="block text-sm font-medium text-gray-700 mb-1">"Filter by Role"</label>
                <select
                    id="role-filter"
                    class="mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md h-10 border"
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

            <div class="flex items-end mt-6">
                <button
                    type="button"
                    class="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 h-10"
                    on:click=move |_| on_clear_filters.call(())
                >
                    "Clear Filters"
                </button>
            </div>
        </div>
    }
}
