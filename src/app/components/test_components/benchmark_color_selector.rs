use leptos::*;

//this component is used to allow a user to select a benchmark category and display its color

const COLOR_PALETTE: &[&str] = &[
    "#10b981", // green
    "#06b6d4", // cyan
    "#f59e0b", // amber
    "#ef4444", // red
    "#8b5cf6", // purple
    "#f43f5e", // rose
    "#6b7280", // gray
];

#[component]
pub fn BenchmarkColorSelector(
    #[prop(into)] current_color: Signal<String>,
    on_color_change: Callback<String>,
) -> impl IntoView {
    let (show_palette, set_show_palette) = create_signal(false);

    view! {
        <button
            class="flex items-center justify-center w-8 h-8 rounded-full border-2 border-gray-300"
            style={move || format!("background-color: {}", current_color.get())}
            on:click=move |_| {
                set_show_palette.update(|show| *show = !*show);
            }
        >
        </button>

        <Show when=move || show_palette()>
            <div class="absolute z-10 p-2 bg-white border rounded shadow-lg">
                <div class="flex flex-wrap gap-2">
                    {COLOR_PALETTE.iter().map(|&color_value| {
                        view! {
                            <button
                                class="w-8 h-8 rounded-full border-2 border-gray-300 hover:border-gray-500"
                                style={format!("background-color: {}", color_value)}
                                on:click={
                                    let color_to_set = color_value.to_string();
                                    move |_| {
                                        set_show_palette.set(false);
                                        on_color_change.call(color_to_set.clone());
                                    }
                                }
                            ></button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </Show>
    }
}
