// src/app/components/test_components/font_controls.rs

use leptos::*;

// Font configuration constants
pub const DEFAULT_QUESTION_FONT_SIZE: &str = "text-4xl";
pub const DEFAULT_ANSWER_FONT_SIZE: &str = "text-lg";
pub const DEFAULT_QUESTION_FONT_FAMILY: &str = "font-sans";
pub const DEFAULT_ANSWER_FONT_FAMILY: &str = "font-sans";

// Font size options
pub const FONT_SIZES: &[(&str, &str)] = &[
    ("text-xs", "Extra Small"),
    ("text-sm", "Small"),
    ("text-base", "Normal"),
    ("text-lg", "Large"),
    ("text-xl", "Extra Large"),
    ("text-2xl", "2X Large"),
    ("text-3xl", "3X Large"),
    ("text-4xl", "4X Large"),
    ("text-5xl", "5X Large"),
    ("text-6xl", "6X Large"),
    ("text-7xl", "7X Large"),
    ("text-8xl", "8X Large"),
    ("text-9xl", "9X Large"),
];

// Font family options
pub const FONT_FAMILIES: &[(&str, &str)] = &[
    ("font-sans", "Sans Serif"),
    ("font-serif", "Serif"),
    ("font-mono", "Monospace"),
    ("font-custom", "Custom"),
];

// Font settings struct for easier management
#[derive(Debug, Clone)]
pub struct FontSettings {
    pub question_font_size: String,
    pub answer_font_size: String,
    pub question_font_family: String,
    pub answer_font_family: String,
    pub question_bold: bool,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            question_font_size: DEFAULT_QUESTION_FONT_SIZE.to_string(),
            answer_font_size: DEFAULT_ANSWER_FONT_SIZE.to_string(),
            question_font_family: DEFAULT_QUESTION_FONT_FAMILY.to_string(),
            answer_font_family: DEFAULT_ANSWER_FONT_FAMILY.to_string(),
            question_bold: true,
        }
    }
}

impl FontSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn get_question_classes(&self) -> String {
        let bold_class = if self.question_bold {
            "font-bold"
        } else {
            "font-normal"
        };
        format!(
            "{} {} {} break-words",
            self.question_font_size, self.question_font_family, bold_class
        )
    }

    pub fn get_answer_classes(&self) -> String {
        format!("{} {}", self.answer_font_size, self.answer_font_family)
    }
}

// Hook for managing font settings
pub fn use_font_settings() -> (ReadSignal<FontSettings>, WriteSignal<FontSettings>) {
    create_signal(FontSettings::default())
}

// Main font controls component
#[component]
pub fn FontControls(
    font_settings: ReadSignal<FontSettings>,
    set_font_settings: WriteSignal<FontSettings>,
) -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);

    view! {
        <div class="relative">
            {/* Toggle Button */}
            <button
                class="flex items-center justify-center w-10 h-10 bg-white border border-gray-200 rounded-lg shadow-sm hover:bg-gray-50 transition-colors"
                on:click=move |_| set_is_open.update(|open| *open = !*open)
                title="Font Settings"
            >
                <FontIcon />
            </button>

            {/* Font Controls Panel */}
            <Show when=move || is_open.get()>
                <div class="absolute top-12 right-0 z-50 bg-white border border-gray-200 rounded-lg shadow-lg p-4 min-w-[280px]">
                    <FontControlsPanel
                        font_settings=font_settings
                        set_font_settings=set_font_settings
                        set_is_open=set_is_open
                    />
                </div>
            </Show>
        </div>
    }
}

// Font controls panel component
#[component]
fn FontControlsPanel(
    font_settings: ReadSignal<FontSettings>,
    set_font_settings: WriteSignal<FontSettings>,
    set_is_open: WriteSignal<bool>,
) -> impl IntoView {
    let update_question_font_size = move |size: String| {
        set_font_settings.update(|settings| {
            settings.question_font_size = size;
        });
    };

    let update_answer_font_size = move |size: String| {
        set_font_settings.update(|settings| {
            settings.answer_font_size = size;
        });
    };

    let update_question_font_family = move |family: String| {
        set_font_settings.update(|settings| {
            settings.question_font_family = family;
        });
    };

    let update_answer_font_family = move |family: String| {
        set_font_settings.update(|settings| {
            settings.answer_font_family = family;
        });
    };

    let reset_settings = move |_| {
        set_font_settings.update(|settings| {
            settings.reset();
        });
    };

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between mb-3">
                <h3 class="text-sm font-semibold text-gray-800">"Font Settings"</h3>
                <button
                    class="text-gray-400 hover:text-gray-600"
                    on:click=move |_| set_is_open.set(false)
                >
                    <CloseIcon />
                </button>
            </div>

            {/* Question Font Size */}
            <FontSizeSelector
                label="Question Size"
                current_value=move || font_settings.get().question_font_size
                on_change=update_question_font_size
            />

            {/* Answer Font Size */}
            <FontSizeSelector
                label="Answer Size"
                current_value=move || font_settings.get().answer_font_size
                on_change=update_answer_font_size
            />

            {/* Question Font Family */}
            <FontFamilySelector
                label="Question Font"
                current_value=move || font_settings.get().question_font_family
                on_change=update_question_font_family
            />

            {/* Answer Font Family */}
            <FontFamilySelector
                label="Answer Font"
                current_value=move || font_settings.get().answer_font_family
                on_change=update_answer_font_family
            />

            {/* Question Bold Toggle */}
            <BoldToggle
                label="Question Bold"
                current_value=move || font_settings.get().question_bold
                on_change=move |value| {
                    set_font_settings.update(|settings| {
                        settings.question_bold = value;
                    });
                }
            />

            {/* Reset Button */}
            <div class="pt-2 border-t border-gray-100">
                <button
                    class="w-full px-3 py-2 text-xs font-medium text-gray-600 border border-gray-200 rounded hover:bg-gray-50 transition-colors"
                    on:click=reset_settings
                >
                    "Reset to Defaults"
                </button>
            </div>
        </div>
    }
}

// Font size selector component
#[component]
fn FontSizeSelector<F>(
    label: &'static str,
    current_value: impl Fn() -> String + 'static,
    on_change: F,
) -> impl IntoView
where
    F: Fn(String) + 'static,
{
    view! {
        <div>
            <label class="block text-xs font-medium text-gray-700 mb-1">{label}</label>
            <select
                class="w-full p-2 text-sm border border-gray-200 rounded focus:ring-blue-500 focus:border-blue-500"
                prop:value=move || current_value()
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    on_change(value);
                }
            >
                {FONT_SIZES.iter().map(|(class, label)| {
                    view! {
                        <option value=*class>{*label}</option>
                    }
                }).collect_view()}
            </select>
        </div>
    }
}

// Font family selector component
#[component]
fn FontFamilySelector<F>(
    label: &'static str,
    current_value: impl Fn() -> String + 'static,
    on_change: F,
) -> impl IntoView
where
    F: Fn(String) + 'static,
{
    view! {
        <div>
            <label class="block text-xs font-medium text-gray-700 mb-1">{label}</label>
            <select
                class="w-full p-2 text-sm border border-gray-200 rounded focus:ring-blue-500 focus:border-blue-500"
                prop:value=move || current_value()
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    on_change(value);
                }
            >
                {FONT_FAMILIES.iter().map(|(class, label)| {
                    view! {
                        <option value=*class>{*label}</option>
                    }
                }).collect_view()}
            </select>
        </div>
    }
}

// Icon components
#[component]
fn FontIcon() -> impl IntoView {
    view! {
        <svg class="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7"></path>
        </svg>
    }
}

#[component]
fn CloseIcon() -> impl IntoView {
    view! {
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
        </svg>
    }
}

#[component]
fn BoldToggle<F>(
    label: &'static str,
    current_value: impl Fn() -> bool + 'static + Clone,
    on_change: F,
) -> impl IntoView
where
    F: Fn(bool) + 'static + Clone,
{
    let current_value_clone = current_value.clone();
    let on_change_clone = on_change.clone();

    view! {
        <div>
            <label class="block text-xs font-medium text-gray-700 mb-2">{label}</label>
            <div class="flex items-center space-x-0">
                <button
                    type="button"
                    class=move || {
                        if current_value() {
                            "flex-1 px-3 py-2 text-xs font-bold bg-blue-500 text-white rounded-l border border-blue-500 transition-colors"
                        } else {
                            "flex-1 px-3 py-2 text-xs font-normal bg-white text-gray-700 border border-gray-300 rounded-l hover:bg-gray-50 transition-colors"
                        }
                    }
                    on:click=move |_| on_change(true)
                >
                    "Bold"
                </button>
                <button
                    type="button"
                    class=move || {
                        if !current_value_clone() {
                            "flex-1 px-3 py-2 text-xs font-normal bg-blue-500 text-white rounded-r border border-blue-500 transition-colors"
                        } else {
                            "flex-1 px-3 py-2 text-xs font-normal bg-white text-gray-700 border border-gray-300 rounded-r hover:bg-gray-50 transition-colors"
                        }
                    }
                    on:click=move |_| on_change_clone(false)
                >
                    "Normal"
                </button>
            </div>
        </div>
    }
}

#[component]
fn BoldIcon() -> impl IntoView {
    view! {
        <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h4.5a3.5 3.5 0 013.5 3.5v.5a3 3 0 01-3 3H6v2h4a3 3 0 110 6H4a1 1 0 01-1-1V4zm2 1v4h3.5a1.5 1.5 0 001.5-1.5v-.5A1.5 1.5 0 008.5 5H5zm0 6v4h4a1 1 0 100-2H6a1 1 0 01-1-1z" clip-rule="evenodd" />
        </svg>
    }
}

#[component]
fn NormalIcon() -> impl IntoView {
    view! {
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
        </svg>
    }
}
