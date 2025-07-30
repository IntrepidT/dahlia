use crate::app::components::data_processing::TestHistoryEntry;
use crate::app::components::overview::overview_table::OverviewTable;
use crate::app::components::overview::search_bar::SearchBar;
use crate::app::components::overview::sort_selector::{SortOption, SortSelector};
use crate::app::components::overview::time_frame_selector::{TimeFrame, TimeFrameSelector};
use leptos::*;

#[component]
pub fn OverviewTab(test_history: Vec<TestHistoryEntry>) -> impl IntoView {
    let (search_query, set_search_query) = create_signal(String::new());
    let (selected_timeframe, set_selected_timeframe) = create_signal(TimeFrame::AllTime);
    let (selected_sort, set_selected_sort) = create_signal(SortOption::DateDesc);

    view! {
        <div class="space-y-6">
            // Header section
            <div class="flex flex-col space-y-4 sm:flex-row sm:items-center sm:justify-between sm:space-y-0">
                <div>
                    <h2 class="text-2xl font-semibold text-gray-900">"Recent Tests"</h2>
                    <p class="mt-1 text-sm text-gray-600">
                        "Track test performance and progress over time"
                    </p>
                </div>
            </div>

            // Controls section
            <div class="flex flex-col space-y-4 sm:flex-row sm:items-center sm:space-y-0 sm:space-x-4">
                <div class="flex-1 max-w-md">
                    <SearchBar
                        search_query=search_query
                        set_search_query=set_search_query
                    />
                </div>
                <div class="flex space-x-3">
                    <TimeFrameSelector
                        selected_timeframe=selected_timeframe
                        set_selected_timeframe=set_selected_timeframe
                    />
                    <SortSelector
                        selected_sort=selected_sort
                        set_selected_sort=set_selected_sort
                    />
                </div>
            </div>

            // Table section
            <OverviewTable
                test_history=test_history
                search_query=search_query
                selected_timeframe=selected_timeframe
                selected_sort=selected_sort
            />
        </div>
    }
}
