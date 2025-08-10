use crate::app::models::auth::SamlConfig;
use crate::app::models::user::UserRole;
use crate::app::server_functions::saml_auth::{
    create_saml_config, delete_saml_config, get_saml_config_details, get_saml_institutions,
    toggle_saml_config, update_saml_config, SamlInstitution,
};
use leptos::prelude::*;
use leptos::prelude::*;

#[component]
pub fn SamlAdminContent(user_id: i64) -> impl IntoView {
    let (institutions, set_institutions) = create_signal::<Vec<SamlInstitution>>(Vec::new());
    let (show_add_form, set_show_add_form) = signal(false);
    let (show_edit_form, set_show_edit_form) = signal(false);
    let (editing_config, set_editing_config) = create_signal::<Option<SamlConfig>>(None);
    let (loading, set_loading) = signal(false);
    let (message, set_message) = create_signal::<Option<(String, bool)>>(None);
    let (show_delete_confirm, set_show_delete_confirm) = signal(false);
    let (delete_target, set_delete_target) = create_signal::<Option<(String, String)>>(None);

    // Form fields for adding/editing SAML config
    let (institution_name, set_institution_name) = signal("".to_string());
    let (entity_id, set_entity_id) = signal("".to_string());
    let (sso_url, set_sso_url) = signal("".to_string());
    let (slo_url, set_slo_url) = signal("".to_string());
    let (x509_cert, set_x509_cert) = signal("".to_string());
    let (metadata_url, set_metadata_url) = signal("".to_string());
    let (config_active, set_config_active) = signal(true);

    // Load institutions on component mount
    let load_institutions = Action::new(move |_: &()| async move {
        match get_saml_institutions().await {
            Ok(institutions_list) => {
                set_institutions.set(institutions_list);
                Ok(())
            }
            Err(e) => {
                log::info!("Failed to load SAML institutions: {:?}", e);
                set_message.set(Some((format!("Failed to load institutions: {}", e), false)));
                Err(e)
            }
        }
    });

    // Load institutions on mount
    Effect::new(move |_| {
        let _ = load_institutions.dispatch(());
    });

    // Clear form helper
    let clear_form = move || {
        set_institution_name.set("".to_string());
        set_entity_id.set("".to_string());
        set_sso_url.set("".to_string());
        set_slo_url.set("".to_string());
        set_x509_cert.set("".to_string());
        set_metadata_url.set("".to_string());
        set_config_active.set(true);
        set_editing_config.set(None);
    };

    // Add configuration action
    let handle_add_config = Action::new(move |_: &()| {
        let institution_name = institution_name.get();
        let entity_id = entity_id.get();
        let sso_url = sso_url.get();
        let slo_url = slo_url.get();
        let x509_cert = x509_cert.get();
        let metadata_url = metadata_url.get();

        async move {
            set_loading.set(true);
            set_message.set(None);

            // Basic validation
            if institution_name.trim().is_empty()
                || entity_id.trim().is_empty()
                || sso_url.trim().is_empty()
                || x509_cert.trim().is_empty()
            {
                set_message.set(Some((
                    "All required fields must be filled".to_string(),
                    false,
                )));
                set_loading.set(false);
                return;
            }

            // Validate URLs
            if !sso_url.starts_with("http") {
                set_message.set(Some((
                    "SSO URL must be a valid HTTP/HTTPS URL".to_string(),
                    false,
                )));
                set_loading.set(false);
                return;
            }

            if !slo_url.is_empty() && !slo_url.starts_with("http") {
                set_message.set(Some((
                    "SLO URL must be a valid HTTP/HTTPS URL".to_string(),
                    false,
                )));
                set_loading.set(false);
                return;
            }

            if !metadata_url.is_empty() && !metadata_url.starts_with("http") {
                set_message.set(Some((
                    "Metadata URL must be a valid HTTP/HTTPS URL".to_string(),
                    false,
                )));
                set_loading.set(false);
                return;
            }

            let slo_url_opt = if slo_url.trim().is_empty() {
                None
            } else {
                Some(slo_url)
            };
            let metadata_url_opt = if metadata_url.trim().is_empty() {
                None
            } else {
                Some(metadata_url)
            };

            match create_saml_config(
                institution_name.clone(),
                entity_id,
                sso_url,
                slo_url_opt,
                x509_cert,
                metadata_url_opt,
            )
            .await
            {
                Ok(response) => {
                    if response.success {
                        set_message.set(Some((response.message, true)));
                        clear_form();
                        set_show_add_form.set(false);
                        load_institutions.dispatch(());
                    } else {
                        set_message.set(Some((response.message, false)));
                    }
                }
                Err(e) => {
                    set_message.set(Some((
                        format!("Failed to create SAML config: {}", e),
                        false,
                    )));
                }
            }

            set_loading.set(false);
        }
    });

    // Edit configuration action
    let handle_edit_config = Action::new(move |config_id: &String| {
        let config_id = config_id.clone();
        async move {
            set_loading.set(true);
            match get_saml_config_details(config_id).await {
                Ok(config) => {
                    set_institution_name.set(config.institution_name.clone());
                    set_entity_id.set(config.entity_id.clone());
                    set_sso_url.set(config.sso_url.clone());
                    set_slo_url.set(config.slo_url.clone().unwrap_or_default());
                    set_x509_cert.set(config.x509_cert.clone());
                    set_config_active.set(config.active);
                    set_editing_config.set(Some(config));
                    set_show_edit_form.set(true);
                    set_show_add_form.set(false);
                }
                Err(e) => {
                    set_message.set(Some((
                        format!("Failed to load configuration: {}", e),
                        false,
                    )));
                }
            }
            set_loading.set(false);
        }
    });

    // Update configuration action
    let handle_update_config = Action::new(move |_: &()| {
        let config = editing_config.get();
        let institution_name = institution_name.get();
        let entity_id = entity_id.get();
        let sso_url = sso_url.get();
        let slo_url = slo_url.get();
        let x509_cert = x509_cert.get();
        let metadata_url = metadata_url.get();
        let active = config_active.get();

        async move {
            if let Some(config) = config {
                set_loading.set(true);
                set_message.set(None);

                // Basic validation
                if institution_name.trim().is_empty()
                    || entity_id.trim().is_empty()
                    || sso_url.trim().is_empty()
                    || x509_cert.trim().is_empty()
                {
                    set_message.set(Some((
                        "All required fields must be filled".to_string(),
                        false,
                    )));
                    set_loading.set(false);
                    return;
                }

                let slo_url_opt = if slo_url.trim().is_empty() {
                    None
                } else {
                    Some(slo_url)
                };
                let metadata_url_opt = if metadata_url.trim().is_empty() {
                    None
                } else {
                    Some(metadata_url)
                };

                match update_saml_config(
                    config.id.to_string(),
                    institution_name,
                    entity_id,
                    sso_url,
                    slo_url_opt,
                    x509_cert,
                    metadata_url_opt,
                    active,
                )
                .await
                {
                    Ok(response) => {
                        if response.success {
                            set_message.set(Some((response.message, true)));
                            clear_form();
                            set_show_edit_form.set(false);
                            load_institutions.dispatch(());
                        } else {
                            set_message.set(Some((response.message, false)));
                        }
                    }
                    Err(e) => {
                        set_message.set(Some((
                            format!("Failed to update SAML config: {}", e),
                            false,
                        )));
                    }
                }

                set_loading.set(false);
            }
        }
    });

    // Delete configuration action
    let handle_delete_config = Action::new(move |config_id: &String| {
        let config_id = config_id.clone();
        async move {
            set_loading.set(true);
            match delete_saml_config(config_id).await {
                Ok(response) => {
                    if response.success {
                        set_message.set(Some((response.message, true)));
                        load_institutions.dispatch(());
                    } else {
                        set_message.set(Some((response.message, false)));
                    }
                }
                Err(e) => {
                    set_message.set(Some((
                        format!("Failed to delete SAML config: {}", e),
                        false,
                    )));
                }
            }
            set_loading.set(false);
        }
    });

    // Toggle configuration status action
    let handle_toggle_config = Action::new(move |config_id: &String| {
        let config_id = config_id.clone();
        async move {
            set_loading.set(true);
            match toggle_saml_config(config_id).await {
                Ok(response) => {
                    if response.success {
                        set_message.set(Some((response.message, true)));
                        load_institutions.dispatch(());
                    } else {
                        set_message.set(Some((response.message, false)));
                    }
                }
                Err(e) => {
                    set_message.set(Some((
                        format!("Failed to toggle SAML config: {}", e),
                        false,
                    )));
                }
            }
            set_loading.set(false);
        }
    });

    view! {
        <div class="space-y-6">
            // Message display
            {move || {
                message.get().map(|(msg, is_success)| {
                    let bg_class = if is_success {
                        "bg-green-900 border-green-700 text-green-200"
                    } else {
                        "bg-red-900 border-red-700 text-red-200"
                    };
                    view! {
                        <div class={format!("border px-4 py-3 rounded {}", bg_class)}>
                            {msg}
                        </div>
                    }
                })
            }}

            // Header with Add button
            <div class="flex justify-between items-center">
                <h4 class="text-lg font-medium text-gray-300">"SAML Institutions"</h4>
                <button
                    class="px-3 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 text-sm disabled:bg-gray-500"
                    on:click=move |_| {
                        if show_edit_form.get() {
                            set_show_edit_form.set(false);
                            clear_form();
                        }
                        set_show_add_form.update(|show| *show = !*show);
                        if show_add_form.get() {
                            clear_form();
                        }
                    }
                    prop:disabled=move || loading.get()
                >
                    {move || if show_add_form.get() { "Cancel" } else { "Add Institution" }}
                </button>
            </div>

            // Add form
            {move || {
                if show_add_form.get() && !show_edit_form.get() {
                    view! {
                        <SamlConfigForm
                            title="Add SAML Configuration"
                            submit_label="Create Configuration"
                            loading=loading
                            institution_name=institution_name
                            set_institution_name=set_institution_name
                            entity_id=entity_id
                            set_entity_id=set_entity_id
                            sso_url=sso_url
                            set_sso_url=set_sso_url
                            slo_url=slo_url
                            set_slo_url=set_slo_url
                            x509_cert=x509_cert
                            set_x509_cert=set_x509_cert
                            metadata_url=metadata_url
                            set_metadata_url=set_metadata_url
                            config_active=config_active
                            set_config_active=set_config_active
                            on_submit=Callback::new(move |_| { let _ = handle_add_config.dispatch(()); })
                            on_cancel=Callback::new(move |_| {
                                set_show_add_form.set(false);
                                clear_form();
                            })
                        />
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            // Edit form
            {move || {
                if show_edit_form.get() {
                    view! {
                        <SamlConfigForm
                            title="Edit SAML Configuration"
                            submit_label="Update Configuration"
                            loading=loading
                            institution_name=institution_name
                            set_institution_name=set_institution_name
                            entity_id=entity_id
                            set_entity_id=set_entity_id
                            sso_url=sso_url
                            set_sso_url=set_sso_url
                            slo_url=slo_url
                            set_slo_url=set_slo_url
                            x509_cert=x509_cert
                            set_x509_cert=set_x509_cert
                            metadata_url=metadata_url
                            set_metadata_url=set_metadata_url
                            config_active=config_active
                            set_config_active=set_config_active
                            on_submit=Callback::new(move |_| { let _ = handle_update_config.dispatch(()); })
                            on_cancel=Callback::new(move |_| {
                                set_show_edit_form.set(false);
                                clear_form();
                            })
                        />
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            // Institutions list
            <div class="space-y-3">
                {move || {
                    let institutions_list = institutions.get();
                    if institutions_list.is_empty() {
                        view! {
                            <div class="p-4 text-center text-gray-400 bg-gray-800 rounded border border-gray-600">
                                <p>"No SAML institutions configured yet."</p>
                                <p class="text-sm mt-1">"Click 'Add Institution' to get started."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-2">
                                {institutions_list.into_iter().map(|institution| {
                                    let institution_id_for_edit = institution.id.clone();
                                    let institution_id_for_delete = institution.id.clone();
                                    let institution_id_for_toggle = institution.id.clone();
                                    let institution_name_for_delete = institution.name.clone();
                                    let institution_name_display = institution.name.clone();
                                    let institution_id_display = institution.id.clone();
                                    let institution_active = institution.active;

                                    view! {
                                        <div class="p-4 bg-gray-800 rounded border border-gray-600 hover:bg-gray-750">
                                            <div class="flex items-center justify-between">
                                                <div>
                                                    <h5 class="font-medium text-gray-200">{institution_name_display}</h5>
                                                    <p class="text-sm text-gray-400">
                                                        "Institution ID: " {institution_id_display}
                                                    </p>
                                                </div>
                                                <div class="flex items-center space-x-2">
                                                    {if institution_active {
                                                        view! {
                                                            <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-900 text-green-200">
                                                                "Active"
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-red-900 text-red-200">
                                                                "Inactive"
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                    <button
                                                        class="text-blue-400 hover:text-blue-300 text-sm disabled:text-gray-500"
                                                        on:click=move |_| {
                                                            let _ = handle_edit_config.dispatch(institution_id_for_edit.clone());
                                                        }
                                                        prop:disabled=move || loading.get()
                                                    >
                                                        "Edit"
                                                    </button>
                                                    <button
                                                        class="text-yellow-400 hover:text-yellow-300 text-sm disabled:text-gray-500"
                                                        on:click=move |_| {
                                                            let _ = handle_toggle_config.dispatch(institution_id_for_toggle.clone());
                                                        }
                                                        prop:disabled=move || loading.get()
                                                    >
                                                        {if institution_active { "Disable" } else { "Enable" }}
                                                    </button>
                                                    <button
                                                        class="text-red-400 hover:text-red-300 text-sm disabled:text-gray-500"
                                                        on:click=move |_| {
                                                            set_delete_target.set(Some((institution_id_for_delete.clone(), institution_name_for_delete.clone())));
                                                            set_show_delete_confirm.set(true);
                                                        }
                                                        prop:disabled=move || loading.get()
                                                    >
                                                        "Delete"
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            // Service Provider Info
            <div class="bg-blue-900 border border-blue-700 rounded p-4">
                <h5 class="text-md font-medium text-blue-200 mb-3">"Service Provider Information"</h5>
                <p class="text-blue-300 mb-3 text-sm">
                    "Provide this information to your identity provider administrators:"
                </p>
                <div class="bg-gray-800 border border-gray-600 rounded p-3 font-mono text-sm">
                    <div class="space-y-1 text-gray-300">
                        <div>
                            <span class="text-blue-300">"Entity ID:"</span>
                            <span class="ml-2">{move || format!("{}/saml/metadata", std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()))}</span>
                        </div>
                        <div>
                            <span class="text-blue-300">"ACS URL:"</span>
                            <span class="ml-2">{move || format!("{}/saml/acs", std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()))}</span>
                        </div>
                        <div>
                            <span class="text-blue-300">"SLS URL:"</span>
                            <span class="ml-2">{move || format!("{}/saml/sls", std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()))}</span>
                        </div>
                    </div>
                </div>
            </div>

            // Delete confirmation modal - moved to correct scope
            {move || {
                if show_delete_confirm.get() {
                    if let Some((config_id, institution_name)) = delete_target.get() {
                        view! {
                            <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                                <div class="bg-gray-800 p-6 rounded-lg border border-gray-600 max-w-md w-full mx-4">
                                    <h3 class="text-lg font-medium text-gray-200 mb-4">"Confirm Deletion"</h3>
                                    <p class="text-gray-300 mb-6">
                                        "Are you sure you want to delete the SAML configuration for '"
                                        <span class="font-semibold">{institution_name}</span>
                                        "'? This action cannot be undone."
                                    </p>
                                    <div class="flex justify-end space-x-3">
                                        <button
                                            class="px-4 py-2 bg-gray-600 text-gray-200 rounded hover:bg-gray-500"
                                            on:click=move |_| {
                                                set_show_delete_confirm.set(false);
                                                set_delete_target.set(None);
                                            }
                                        >
                                            "Cancel"
                                        </button>
                                        <button
                                            class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
                                            on:click=move |_| {
                                                handle_delete_config.dispatch(config_id.clone());
                                                set_show_delete_confirm.set(false);
                                                set_delete_target.set(None);
                                            }
                                        >
                                            "Delete"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn SamlConfigForm(
    #[prop(into)] title: String,
    #[prop(into)] submit_label: String,
    #[prop(into)] loading: ReadSignal<bool>,
    #[prop(into)] institution_name: ReadSignal<String>,
    #[prop(into)] set_institution_name: WriteSignal<String>,
    #[prop(into)] entity_id: ReadSignal<String>,
    #[prop(into)] set_entity_id: WriteSignal<String>,
    #[prop(into)] sso_url: ReadSignal<String>,
    #[prop(into)] set_sso_url: WriteSignal<String>,
    #[prop(into)] slo_url: ReadSignal<String>,
    #[prop(into)] set_slo_url: WriteSignal<String>,
    #[prop(into)] x509_cert: ReadSignal<String>,
    #[prop(into)] set_x509_cert: WriteSignal<String>,
    #[prop(into)] metadata_url: ReadSignal<String>,
    #[prop(into)] set_metadata_url: WriteSignal<String>,
    #[prop(into)] config_active: ReadSignal<bool>,
    #[prop(into)] set_config_active: WriteSignal<bool>,
    #[prop(into)] on_submit: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="bg-gray-800 p-4 rounded border border-gray-600">
            <h5 class="text-md font-medium text-gray-200 mb-3">{title}</h5>
            <form on:submit=move |ev| {
                ev.prevent_default();
                if !loading.get() {
                    on_submit.run(());
                }
            }>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1">
                            "Institution Name" <span class="text-red-400">"*"</span>
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || institution_name.get()
                            on:input=move |ev| set_institution_name.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                            placeholder="University of Example"
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1">
                            "Entity ID" <span class="text-red-400">"*"</span>
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || entity_id.get()
                            on:input=move |ev| set_entity_id.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                            placeholder="https://idp.example.edu/saml/metadata"
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1">
                            "SSO URL" <span class="text-red-400">"*"</span>
                        </label>
                        <input
                            type="url"
                            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || sso_url.get()
                            on:input=move |ev| set_sso_url.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                            placeholder="https://idp.example.edu/saml/sso"
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1">
                            "SLO URL (Optional)"
                        </label>
                        <input
                            type="url"
                            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || slo_url.get()
                            on:input=move |ev| set_slo_url.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                            placeholder="https://idp.example.edu/saml/slo"
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1">
                            "Metadata URL (Optional)"
                        </label>
                        <input
                            type="url"
                            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || metadata_url.get()
                            on:input=move |ev| set_metadata_url.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                            placeholder="https://idp.example.edu/saml/metadata"
                        />
                    </div>

                    <div class="flex items-center">
                        <label class="flex items-center text-sm font-medium text-gray-300">
                            <input
                                type="checkbox"
                                class="mr-2 rounded bg-gray-700 border-gray-600 text-blue-600 focus:ring-blue-500"
                                prop:checked=move || config_active.get()
                                on:change=move |ev| set_config_active.set(event_target_checked(&ev))
                                prop:disabled=move || loading.get()
                            />
                            "Active Configuration"
                        </label>
                    </div>
                </div>

                <div class="mt-4">
                    <label class="block text-sm font-medium text-gray-300 mb-1">
                        "X.509 Certificate" <span class="text-red-400">"*"</span>
                    </label>
                    <textarea
                        rows="4"
                        class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
                        prop:value=move || x509_cert.get()
                        on:input=move |ev| set_x509_cert.set(event_target_value(&ev))
                        prop:disabled=move || loading.get()
                        placeholder="-----BEGIN CERTIFICATE-----
MIIEbzCCA1egAwIBAgIJAIYhQeZPzfH3MA0GCSqGSIb3DQEBCwUAMIGBMQswCQYD...
-----END CERTIFICATE-----"
                    ></textarea>
                </div>

                <div class="mt-4 flex justify-end space-x-3">
                    <button
                        type="button"
                        class="px-3 py-2 bg-gray-600 text-gray-200 rounded hover:bg-gray-500 text-sm disabled:bg-gray-500"
                        on:click=move |_| on_cancel.run(())
                        prop:disabled=move || loading.get()
                    >
                        "Cancel"
                    </button>
                    <button
                        type="submit"
                        class="px-3 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-500 disabled:cursor-not-allowed text-sm"
                        prop:disabled=move || loading.get()
                    >
                        {move || {
                            if loading.get() {
                                "Processing...".to_string()
                            } else {
                                submit_label.clone()
                            }
                        }}
                    </button>
                </div>
            </form>
        </div>
    }
}
