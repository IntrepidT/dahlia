use crate::app::models::global::SettingsCache;
use crate::app::server_functions::globals::get_global_settings;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn SettingsProvider(children: ChildrenFn) -> impl IntoView {
    let (settings, set_settings) = signal(SettingsCache::default());
    let (loading, set_loading) = signal(true);

    // Load settings on mount
    Effect::new(move |_| {
        set_loading.set(true);

        spawn_local(async move {
            match get_global_settings().await {
                Ok(settings_data) => {
                    log::info!("Settings loaded: {:?}", settings_data);
                    set_settings.set(settings_data);
                }
                Err(err) => {
                    log::info!("Error loading settings: {:?}", err);
                    // Keep default settings on error
                }
            }
            set_loading.set(false);
        });
    });

    // Provide settings context as tuple
    provide_context((settings, set_settings));
    provide_context(loading);

    children()
}

// Component to conditionally render based on settings
#[component]
pub fn ConditionalRender(
    #[prop(default = false)] student_protections_required: bool,
    children: ChildrenFn,
    #[prop(optional)] fallback: Option<ChildrenFn>,
) -> impl IntoView {
    let (settings, _) = use_settings();
    let loading = use_settings_loading();

    view! {
        <Show
            when=move || !loading.get()
            fallback=move || view! { <div>"Loading settings..."</div> }
        >
            {children()}
        </Show>
    }
}

// Safe hook that returns Option instead of panicking
pub fn try_use_settings() -> Option<(ReadSignal<SettingsCache>, WriteSignal<SettingsCache>)> {
    use_context::<(ReadSignal<SettingsCache>, WriteSignal<SettingsCache>)>()
}

// Updated hook with better error handling
pub fn use_settings() -> (ReadSignal<SettingsCache>, WriteSignal<SettingsCache>) {
    use_context::<(ReadSignal<SettingsCache>, WriteSignal<SettingsCache>)>()
        .expect("Settings context not found - make sure SettingsProvider wraps your app and the component is mounted after settings load")
}

// Add the missing function - returns ReadSignal directly, panics if not found
pub fn use_settings_loading() -> ReadSignal<bool> {
    use_context::<ReadSignal<bool>>()
        .expect("Settings loading context not found - make sure SettingsProvider wraps your app")
}

// Safe version that returns Option
pub fn try_use_settings_loading() -> Option<ReadSignal<bool>> {
    use_context::<ReadSignal<bool>>()
}

// Hook that waits for settings to be loaded - FIXED
pub fn use_settings_when_ready() -> Option<(ReadSignal<SettingsCache>, WriteSignal<SettingsCache>)>
{
    let settings = try_use_settings()?;
    let loading = try_use_settings_loading()?; // Use the safe version

    if loading.get() {
        // Now we can call .get() on the ReadSignal<bool>
        None
    } else {
        Some(settings)
    }
}

// Safe version of use_student_protections
pub fn use_student_protections() -> bool {
    match try_use_settings() {
        Some((settings, _)) => settings.get().student_protections,
        None => {
            log::info!("Settings context not available, defaulting student_protections to false");
            false
        }
    }
}

// Component wrapper that ensures settings are loaded - FIXED
#[component]
pub fn WithSettings(children: ChildrenFn) -> impl IntoView {
    let loading = use_context::<ReadSignal<bool>>();
    let settings_context = try_use_settings();

    view! {
        <Show
            when=move || {
                match (loading, settings_context) {
                    (Some(loading), Some(_)) if !loading.get() => true,
                    _ => false,
                }
            }
            fallback=move || {
                match (loading, settings_context) {
                    (Some(_), Some(_)) => {
                        // Settings context exists but still loading
                        view! { <div class="text-gray-400">"Loading settings..."</div> }.into_any()
                    }
                    _ => {
                        // Settings context not available
                        view! { <div class="text-red-400">"Settings not available"</div> }.into_any()
                    }
                }
            }
        >
            {children()}
        </Show>
    }
}
