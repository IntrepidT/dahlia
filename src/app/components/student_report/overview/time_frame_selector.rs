use leptos::*;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum TimeFrame {
    LastWeek,
    LastMonth,
    Last3Months,
    LastYear,
    AllTime,
}

impl TimeFrame {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeFrame::LastWeek => "Last 7 days",
            TimeFrame::LastMonth => "Last 30 days",
            TimeFrame::Last3Months => "Last 90 days",
            TimeFrame::LastYear => "Last year",
            TimeFrame::AllTime => "All time",
        }
    }

    pub fn as_value(&self) -> &'static str {
        match self {
            TimeFrame::LastWeek => "7",
            TimeFrame::LastMonth => "30",
            TimeFrame::Last3Months => "90",
            TimeFrame::LastYear => "365",
            TimeFrame::AllTime => "all",
        }
    }

    pub fn from_value(value: &str) -> Self {
        match value {
            "7" => TimeFrame::LastWeek,
            "30" => TimeFrame::LastMonth,
            "90" => TimeFrame::Last3Months,
            "365" => TimeFrame::LastYear,
            _ => TimeFrame::AllTime,
        }
    }

    pub fn all_options() -> Vec<TimeFrame> {
        vec![
            TimeFrame::LastWeek,
            TimeFrame::LastMonth,
            TimeFrame::Last3Months,
            TimeFrame::LastYear,
            TimeFrame::AllTime,
        ]
    }
}

#[component]
pub fn TimeFrameSelector(
    #[prop(into)] selected_timeframe: ReadSignal<TimeFrame>,
    #[prop(into)] set_selected_timeframe: WriteSignal<TimeFrame>,
) -> impl IntoView {
    view! {
        <div class="relative">
            <select
                class="appearance-none bg-white border border-gray-200 rounded-lg px-4 py-2.5 pr-8 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200 cursor-pointer"
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    set_selected_timeframe(TimeFrame::from_value(&value));
                }
            >
                {TimeFrame::all_options().into_iter().map(|option| {
                    let is_selected = move || selected_timeframe.get() == option;
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
