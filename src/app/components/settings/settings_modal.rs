use crate::app::components::settings::bulk_enrollment_modal::BulkUploadModal;
use crate::app::middleware::global_settings::{try_use_settings, try_use_settings_loading};
use crate::app::models::global::{GlobalSetting, SettingsCache};
use crate::app::models::setting_data::UserSettings;
use crate::app::models::user::UserJwt;
use crate::app::server_functions::globals::{
    get_global_settings, restore_student_ids_from_file, toggle_student_protection,
};
use crate::app::server_functions::user_settings::{
    get_user_settings, update_dark_mode, update_pinned_sidebar,
};
use leptos::*;
#[cfg(feature = "hydrate")]
use {
    wasm_bindgen::closure::Closure,
    wasm_bindgen::JsCast,
    web_sys::{Event, FileList, HtmlInputElement},
};

#[component]
pub fn SettingsModal(
    #[prop(into)] show: ReadSignal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] user_id: i64,
) -> impl IntoView {
    let (selected_tab, set_selected_tab) = create_signal("General".to_string());

    // Load user settings
    let user_settings_resource = create_resource(
        move || user_id,
        |user_id| async move { get_user_settings(user_id).await },
    );

    view! {
        <Show when=move || show.get()>
            // Modal Backdrop
            <div class="fixed inset-0 bg-black bg-opacity-60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
                // Modal Container
                <div class="bg-gray-800 rounded-lg shadow-2xl w-full max-w-5xl h-5/6 flex overflow-hidden border border-gray-700">

                    // Sidebar
                    <div class="w-64 bg-gray-900 border-r border-gray-700 flex flex-col">
                        // Header
                        <div class="p-4 border-b border-gray-700 flex justify-between items-center">
                            <h2 class="text-lg font-semibold text-gray-100">"Settings"</h2>
                            <button
                                class="text-gray-400 hover:text-gray-200 text-xl leading-none"
                                on:click=move |_| on_close.call(())
                            >
                                "×"
                            </button>
                        </div>

                        // Navigation
                        <div class="flex-1 overflow-y-auto">
                            <div class="p-2">
                                // Core Category
                                <div class="mb-4">
                                    <div class="text-xs uppercase text-gray-500 font-semibold mb-2 px-2">"Core (beta)"</div>
                                    <nav class="space-y-1">
                                        <SettingsNavButton
                                            label="General"
                                            selected=selected_tab
                                            on_select=set_selected_tab
                                        />
                                        <SettingsNavButton
                                            label="Editor"
                                            selected=selected_tab
                                            on_select=set_selected_tab
                                        />
                                        <SettingsNavButton
                                            label="Files & Links"
                                            selected=selected_tab
                                            on_select=set_selected_tab
                                        />
                                        <SettingsNavButton
                                            label="Appearance"
                                            selected=selected_tab
                                            on_select=set_selected_tab
                                        />
                                    </nav>
                                </div>

                                // Plugins Category
                                <div class="mb-4">
                                    <div class="text-xs uppercase text-gray-500 font-semibold mb-2 px-2">"Admin Settings (beta)"</div>
                                    <nav class="space-y-1">
                                        <SettingsNavButton
                                            label="School-wide Settings"
                                            selected=selected_tab
                                            on_select=set_selected_tab
                                        />
                                        <SettingsNavButton
                                            label="Promote Students"
                                            selected=selected_tab
                                            on_select=set_selected_tab
                                        />
                                        <SettingsNavButton
                                            label="Daily notes"
                                            selected=selected_tab
                                            on_select=set_selected_tab
                                        />
                                    </nav>
                                </div>

                                // Advanced Category
                                <div>
                                    <div class="text-xs uppercase text-gray-500 font-semibold mb-2 px-2">"Advanced (beta)"</div>
                                    <nav class="space-y-1">
                                        <SettingsNavButton
                                            label="Hotkeys"
                                            selected=selected_tab
                                            on_select=set_selected_tab
                                        />
                                        <SettingsNavButton
                                            label="Developer Settings"
                                            selected=selected_tab
                                            on_select=set_selected_tab
                                        />
                                        <SettingsNavButton
                                            label="About"
                                            selected=selected_tab
                                            on_select=set_selected_tab
                                        />
                                    </nav>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Main Content Area
                    <div class="flex-1 flex flex-col">
                        // Content Header
                        <div class="p-6 border-b border-gray-700">
                            <h3 class="text-xl font-semibold text-gray-100">{move || selected_tab.get()}</h3>
                        </div>

                        // Content Body
                        <div class="flex-1 p-6 overflow-y-auto">
                            <Suspense fallback=move || view! {
                                <div class="flex items-center justify-center h-32">
                                    <div class="text-gray-400">"Loading settings..."</div>
                                </div>
                            }>
                                {move || {
                                    user_settings_resource.get().map(|settings_result| {
                                        match settings_result {
                                            Ok(settings) => view! {
                                                <SettingsContent
                                                    selected_tab=selected_tab
                                                    user_settings=settings
                                                    user_id=user_id
                                                />
                                            }.into_view(),
                                            Err(_) => view! {
                                                <div class="text-red-400">"Error loading settings"</div>
                                            }.into_view()
                                        }
                                    })
                                }}
                            </Suspense>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn SettingsNavButton(
    #[prop(into)] label: String,
    selected: ReadSignal<String>,
    on_select: WriteSignal<String>,
) -> impl IntoView {
    let label_clone = label.clone();
    let is_selected = move || selected.get() == label_clone;

    view! {
        <button
            class=move || format!(
                "w-full text-left px-3 py-2 rounded text-sm transition-colors {}",
                if is_selected() {
                    "bg-gray-800 text-white"
                } else {
                    "text-gray-300 hover:bg-gray-800 hover:text-white"
                }
            )
            on:click={
                let label = label.clone();
                move |_| on_select.set(label.clone())
            }
        >
            {label}
        </button>
    }
}

#[component]
fn SettingsContent(
    selected_tab: ReadSignal<String>,
    user_settings: UserSettings,
    user_id: i64,
) -> impl IntoView {
    let user = use_context::<ReadSignal<Option<UserJwt>>>().expect("AuthProvider not Found");
    let (show_bulk_upload_modal, set_show_bulk_upload_modal) = create_signal(false);
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);

    // Create reactive signals for settings that sync with server
    let (dark_mode, set_dark_mode) = create_signal(user_settings.ui.dark_mode);
    let (pin_sidebar, set_pin_sidebar) = create_signal(user_settings.ui.pinned_sidebar);

    // Server actions for updating settings
    let update_dark_mode_action = create_action(move |&new_value: &bool| async move {
        match update_dark_mode(user_id, new_value).await {
            Ok(_) => {
                set_dark_mode.set(new_value);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to update dark mode: {}", e);
                Err(e)
            }
        }
    });

    let update_pin_sidebar_action = create_action(move |&new_value: &bool| async move {
        match update_pinned_sidebar(user_id, new_value).await {
            Ok(_) => {
                set_pin_sidebar.set(new_value);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to update pinned sidebar: {}", e);
                Err(e)
            }
        }
    });

    view! {
        <div class="space-y-6">
            {move || match selected_tab.get().as_str() {
                "General" => view! {
                    <div class="space-y-4">
                        <SettingsSection title="Language">
                            <SettingsButton label="English" />
                            <SettingsButton label="Español" />
                            <SettingsButton label="Français" />
                        </SettingsSection>
                        <SettingsSection title="Startup">
                            <SettingsButton label="Open last vault" />
                            <SettingsButton label="Show welcome screen" />
                        </SettingsSection>
                        <SettingsSection title="Preferences">
                            <ToggleSwitch
                                label="Permanently pin sidebar"
                                checked=pin_sidebar
                                on_toggle=Callback::new(move |value| {
                                    update_pin_sidebar_action.dispatch(value);
                                })
                                description="Keep the sidebar closed always until setting turned off"
                            />
                        </SettingsSection>
                    </div>
                }.into_view(),

                "Editor" => view! {
                    <div class="space-y-4">
                        <SettingsSection title="Display">
                            <SettingsButton label="Show line numbers" />
                            <SettingsButton label="Word wrap" />
                            <SettingsButton label="Show frontmatter" />
                        </SettingsSection>
                        <SettingsSection title="Behavior">
                            <SettingsButton label="Auto-save" />
                            <SettingsButton label="Vim key bindings" />
                        </SettingsSection>
                    </div>
                }.into_view(),

                "Files & Links" => view! {
                    <div class="space-y-4">
                        <SettingsSection title="Files">
                            <SettingsButton label="Confirm file deletion" />
                            <SettingsButton label="Always update internal links" />
                            <SettingsButton label="Use [[Wikilinks]]" />
                        </SettingsSection>
                        <SettingsSection title="Attachments">
                            <SettingsButton label="Attachment folder path" />
                            <SettingsButton label="Automatically update internal links" />
                        </SettingsSection>
                    </div>
                }.into_view(),

                "Appearance" => view! {
                    <div class="space-y-4">
                        <SettingsSection title="Theme">
                            <ToggleSwitch
                                label="Dark mode"
                                checked=dark_mode
                                on_toggle=Callback::new(move |value| {
                                    update_dark_mode_action.dispatch(value);
                                })
                                description="Toggle dark mode theme"
                            />
                            <SettingsButton label="Light mode" />
                            <SettingsButton label="System default" />
                        </SettingsSection>
                        <SettingsSection title="Interface">
                            <SettingsButton label="Show tab bar" />
                            <SettingsButton label="Show status bar" />
                        </SettingsSection>
                    </div>
                }.into_view(),

                "Promote Students" => view! {
                    <div class="space-y-4">
                        <SettingsSection title="Promote Students">
                            <button
                                on:click=move |_| set_show_bulk_upload_modal(true)
                            >
                                <SettingsButton label="Promote students in bulk" />
                            </button>
                            <SettingsButton label="Promote students manually" navigate=true />
                        </SettingsSection>

                        <Show when=move || show_bulk_upload_modal()>
                            <BulkUploadModal
                                set_show_modal=set_show_bulk_upload_modal
                                set_refresh_trigger=set_refresh_trigger
                            />
                        </Show>
                    </div>
                }.into_view(),

                "Developer Settings" => view! {
                    <div class="space-y-4">
                        <SettingsSection title="Developer Settings">
                            <Show when=move || user.get().map(|u| u.is_super_admin()).unwrap_or(false)>
                                <StudentProtectionToggleSafe />
                            </Show>
                        </SettingsSection>
                    </div>
                }.into_view(),

                _ => view! {
                    <div class="space-y-4">
                        <SettingsSection title=&selected_tab.get()>
                            <SettingsButton label="Option 1" />
                            <SettingsButton label="Option 2" />
                            <SettingsButton label="Option 3" />
                        </SettingsSection>
                    </div>
                }.into_view(),
            }}
        </div>
    }
}

#[component]
fn SettingsSection(#[prop(into)] title: String, children: Children) -> impl IntoView {
    view! {
        <div class="space-y-3">
            <h4 class="text-sm font-medium text-gray-300 uppercase tracking-wide">{title}</h4>
            <div class="space-y-2">
                {children()}
            </div>
        </div>
    }
}

#[component]
fn SettingsButton(
    #[prop(into)] label: String,
    #[prop(into, optional)] navigate: bool,
) -> impl IntoView {
    view! {
        <button class="w-full flex items-center justify-between px-4 py-3 bg-gray-700 hover:bg-gray-600 rounded border border-gray-600 hover:border-gray-500 transition-colors text-gray-200 hover:text-white">
            <span>{label}</span>
            {if navigate {
                view! {
                    <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
                    </svg>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </button>
    }
}

#[component]
fn ToggleSwitch(
    #[prop(into)] label: String,
    #[prop(into)] checked: ReadSignal<bool>,
    #[prop(into)] on_toggle: Callback<bool>,
    #[prop(into, optional)] description: Option<String>,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between py-3 px-4 bg-gray-700 hover:bg-gray-600 rounded border border-gray-600 hover:border-gray-500 transition-colors">
            <div class="flex-1">
                <div class="text-gray-200 font-medium">{label}</div>
                {description.map(|desc| view! {
                    <div class="text-sm text-gray-400 mt-1">{desc}</div>
                })}
            </div>

            <button
                class=move || format!(
                    "relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-700 {}", if checked.get() { "bg-blue-600" } else { "bg-gray-500"}
                )
                on:click=move |_| on_toggle.call(!checked.get())
            >
                <span
                    class=move || format!(
                        "inline-block h-4 w-4 transform rounded-full bg-white transition {}",
                        if checked.get() {"translate-x-6"} else {"translate-x-1"}
                    )
                />
            </button>
        </div>
    }
}

#[component]
fn StudentProtectionToggleInner(settings: ReadSignal<SettingsCache>) -> impl IntoView {
    let (show_key_modal, set_show_key_modal) = create_signal(false);
    let (show_file_modal, set_show_file_modal) = create_signal(false); // NEW
    let (mapping_key, set_mapping_key) = create_signal(String::new());
    let (selected_file, set_selected_file) = create_signal(Option::<String>::None); // NEW
    let (is_processing, set_is_processing) = create_signal(false);
    let (status_message, set_status_message) = create_signal(Option::<String>::None);

    // Create a local signal to track the toggle state that can be updated
    let (protection_enabled, set_protection_enabled) =
        create_signal(settings.get().student_protections);

    // Update local state when settings change
    create_effect(move |_| {
        set_protection_enabled.set(settings.get().student_protections);
    });

    let toggle_protection_action = create_action(move |(enable, key): &(bool, Option<String>)| {
        let enable = *enable;
        let key = key.clone();
        async move {
            set_is_processing.set(true);
            set_status_message.set(None);

            match toggle_student_protection(enable, key).await {
                Ok(message) => {
                    set_status_message.set(Some(message.clone()));
                    set_protection_enabled.set(enable);

                    if !enable && message.contains("mapping") {
                        set_status_message.set(Some(format!("{}\n\nPlease check your downloads folder or server logs for the mapping file location.", message)));
                    }
                }
                Err(e) => {
                    set_status_message.set(Some(format!("Error: {}", e)));
                }
            }

            set_is_processing.set(false);
            set_show_key_modal.set(false);
            set_mapping_key.set(String::new());
        }
    });

    // NEW: File upload action
    let restore_from_file_action = create_action(move |file_content: &String| {
        let file_content = file_content.clone();
        async move {
            set_is_processing.set(true);
            set_status_message.set(None);

            match restore_student_ids_from_file(file_content).await {
                Ok(message) => {
                    set_status_message.set(Some(message));
                    set_protection_enabled.set(false);
                }
                Err(e) => {
                    set_status_message.set(Some(format!("Error: {}", e)));
                }
            }

            set_is_processing.set(false);
            set_show_file_modal.set(false);
            set_selected_file.set(None);
        }
    });

    // File change handler with better error handling
    #[cfg(feature = "hydrate")]
    let handle_file_change = move |event: Event| {
        let input = event
            .target()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        if let Some(files) = input.files() {
            if files.length() > 0 {
                if let Some(file) = files.get(0) {
                    let file_name = file.name();

                    // Validate file type
                    if !file_name.ends_with(".csv") {
                        set_status_message.set(Some("Error: Please select a CSV file".to_string()));
                        return;
                    }

                    set_selected_file.set(Some(file_name.clone()));
                    set_status_message.set(Some(format!("Selected file: {}", file_name)));

                    // Read file content
                    let file_reader = web_sys::FileReader::new().unwrap();
                    let reader_clone = file_reader.clone();

                    let onload = Closure::wrap(Box::new(move |_: Event| {
                        if let Ok(content) = reader_clone.result() {
                            if let Some(text) = content.as_string() {
                                // Validate CSV content has expected headers
                                let lines: Vec<&str> = text.lines().collect();
                                if lines.is_empty() {
                                    set_status_message
                                        .set(Some("Error: CSV file is empty".to_string()));
                                    return;
                                }

                                let header = lines[0].to_lowercase();
                                if !header.contains("app_id") || !header.contains("student_id") {
                                    set_status_message.set(Some("Error: CSV file must contain 'app_id' and 'student_id' columns".to_string()));
                                    return;
                                }

                                if lines.len() < 2 {
                                    set_status_message.set(Some(
                                        "Error: CSV file contains no data rows".to_string(),
                                    ));
                                    return;
                                }

                                set_status_message.set(Some(format!(
                                    "File validated. Found {} data rows. Ready to restore.",
                                    lines.len() - 1
                                )));
                                restore_from_file_action.dispatch(text);
                            }
                        }
                    }) as Box<dyn FnMut(_)>);

                    file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();

                    let onerror = Closure::wrap(Box::new(move |_: Event| {
                        set_status_message.set(Some("Error: Failed to read file".to_string()));
                    }) as Box<dyn FnMut(_)>);

                    file_reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                    onerror.forget();

                    let _ = file_reader.read_as_text(&file);
                }
            }
        }
    };

    let handle_toggle = move |enable: bool| {
        if enable {
            toggle_protection_action.dispatch((true, None));
        } else {
            set_show_key_modal.set(true);
        }
    };

    view! {
        <div class="space-y-4">
            <div class="p-4 bg-red-900 border border-red-700 rounded-lg">
                <div class="flex items-start space-x-3">
                    <svg class="w-6 h-6 text-red-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.728-.833-2.498 0L3.316 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
                    </svg>
                    <div>
                        <h3 class="text-lg font-medium text-red-200">"DANGER ZONE"</h3>
                        <p class="text-sm text-red-300 mt-1">
                            "Student Data Protection Mode will replace all student IDs with anonymized app IDs. This operation affects the entire database and requires a mapping key to restore."
                        </p>
                    </div>
                </div>
            </div>

            <ToggleSwitch
                label="Student Data Protection Mode"
                checked=protection_enabled
                on_toggle=Callback::new(move |value| {
                    handle_toggle(value);
                })
                description="When enabled, replaces student IDs with anonymized app IDs"
            />

            // NEW: File upload button (only show when protection is enabled)
            <Show when=move || protection_enabled.get()>
                <div class="p-4 bg-blue-900 border border-blue-700 rounded-lg">
                    <h4 class="text-blue-200 font-medium mb-2">"Restore from Mapping File"</h4>
                    <p class="text-sm text-blue-300 mb-3">
                        "Upload the CSV mapping file that was exported when protection was enabled to restore original student IDs."
                    </p>
                    <button
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded text-white transition-colors"
                        on:click=move |_| set_show_file_modal.set(true)
                    >
                        "Upload Mapping File"
                    </button>
                </div>
            </Show>

            <Show when=move || status_message.get().is_some()>
                <div class=move || {
                    let msg = status_message.get().unwrap_or_default();
                    if msg.starts_with("Error:") {
                        "p-3 bg-red-900 border border-red-700 rounded text-red-200 text-sm whitespace-pre-line"
                    } else {
                        "p-3 bg-green-900 border border-green-700 rounded text-green-200 text-sm whitespace-pre-line"
                    }
                }>
                    {move || status_message.get().unwrap_or_default()}
                </div>
            </Show>

            <Show when=move || is_processing.get()>
                <div class="p-3 bg-blue-900 border border-blue-700 rounded text-blue-200 text-sm">
                    "Processing... This may take a few moments."
                </div>
            </Show>

            <Show when=move || show_file_modal.get()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div class="bg-gray-800 p-6 rounded-lg border border-gray-700 max-w-md w-full mx-4">
                        <h3 class="text-lg font-semibold text-gray-100 mb-4">
                            "Upload Mapping File"
                        </h3>
                        <p class="text-sm text-gray-300 mb-4">
                            "Select the CSV file that was exported when student protection was enabled (student_id_mapping.csv)."
                        </p>

                        <div class="mb-4">
                            {
                                #[cfg(feature = "hydrate")]
                                {
                                    view! {
                                        <input
                                            type="file"
                                            accept=".csv"
                                            class="hidden"
                                            id="mapping-file-input"
                                            on:change=handle_file_change
                                        />
                                    }
                                }
                            }
                            <label
                                for="mapping-file-input"
                                class="block w-full p-3 border-2 border-dashed border-gray-600 rounded-lg text-center cursor-pointer hover:border-gray-500 transition-colors"
                            >
                                <svg class="w-8 h-8 text-gray-400 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
                                </svg>
                                <span class="text-gray-300">
                                    {move || selected_file.get().unwrap_or_else(|| "Click to select CSV file".to_string())}
                                </span>
                            </label>
                        </div>

                        <div class="flex justify-end space-x-3">
                            <button
                                class="px-4 py-2 bg-gray-600 hover:bg-gray-500 rounded text-gray-100 transition-colors"
                                on:click=move |_| {
                                    set_show_file_modal.set(false);
                                    set_selected_file.set(None);
                                }
                            >
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn StudentProtectionToggleSafe() -> impl IntoView {
    // Try to get contexts safely
    let settings_context = try_use_settings();
    let loading_context = try_use_settings_loading();

    // Handle the case where contexts are not available
    match (settings_context, loading_context) {
        (Some((settings, _)), Some(loading)) => view! {
            {move || {
                if loading.get() {
                    view! {
                        <div class="text-gray-400">"Loading protection settings..."</div>
                    }.into_view()
                } else {
                    view! {
                        <StudentProtectionToggleInner settings=settings />
                    }.into_view()
                }
            }}
        }
        .into_view(),
        _ => view! {
            <div class="text-red-400">
                "Settings context not available. Make sure SettingsProvider wraps this component."
            </div>
        }
        .into_view(),
    }
}
