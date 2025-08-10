use leptos::prelude::*;
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum SortOption {
    DateDesc,
    DateAsc,
    ScoreDesc,
    ScoreAsc,
    TestNameAsc,
    TestNameDesc,
}

impl SortOption {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOption::DateDesc => "Newest first",
            SortOption::DateAsc => "Oldest first",
            SortOption::ScoreDesc => "Highest score",
            SortOption::ScoreAsc => "Lowest score",
            SortOption::TestNameAsc => "Test name A-Z",
            SortOption::TestNameDesc => "Test name Z-A",
        }
    }

    pub fn as_value(&self) -> &'static str {
        match self {
            SortOption::DateDesc => "date_desc",
            SortOption::DateAsc => "date_asc",
            SortOption::ScoreDesc => "score_desc",
            SortOption::ScoreAsc => "score_asc",
            SortOption::TestNameAsc => "name_asc",
            SortOption::TestNameDesc => "name_desc",
        }
    }

    pub fn from_value(value: &str) -> Self {
        match value {
            "date_asc" => SortOption::DateAsc,
            "score_desc" => SortOption::ScoreDesc,
            "score_asc" => SortOption::ScoreAsc,
            "name_asc" => SortOption::TestNameAsc,
            "name_desc" => SortOption::TestNameDesc,
            _ => SortOption::DateDesc,
        }
    }

    pub fn all_options() -> Vec<SortOption> {
        vec![
            SortOption::DateDesc,
            SortOption::DateAsc,
            SortOption::ScoreDesc,
            SortOption::ScoreAsc,
            SortOption::TestNameAsc,
            SortOption::TestNameDesc,
        ]
    }
}

#[component]
pub fn SortSelector(
    #[prop(into)] selected_sort: ReadSignal<SortOption>,
    #[prop(into)] set_selected_sort: WriteSignal<SortOption>,
) -> impl IntoView {
    view! {
        <div class="relative">
            <select
                class="appearance-none bg-white border border-gray-200 rounded-lg px-4 py-2.5 pr-8 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200 cursor-pointer"
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    set_selected_sort(SortOption::from_value(&value));
                }
            >
                {SortOption::all_options().into_iter().map(|option| {
                    let is_selected = move || selected_sort.get() == option;
                    view! {
                        <option
                            value={option.as_value()}
                            selected=is_selected
                        >
                            {option.as_str()}
                        </option>
                    }
                }).collect::<Vec<_>>()}
            </select>
            <div class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
                <svg class="h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
            </div>
        </div>
    }
}
