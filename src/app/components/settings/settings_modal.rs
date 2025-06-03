use crate::app::components::settings::bulk_enrollment_modal::BulkUploadModal;
use crate::app::models::setting_data::UserSettings;
use crate::app::models::user::UserJwt;
use crate::app::server_functions::user_settings::{
    get_user_settings, update_dark_mode, update_pinned_sidebar,
};
use leptos::*;

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
                                            label="Developer"
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
