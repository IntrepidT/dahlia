use crate::app::models::user::{SessionUser, UserRole};
use crate::app::server_functions::saml_auth::{
    create_saml_config, get_saml_institutions, SamlInstitution,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn SamlAdminPanel() -> impl IntoView {
    let current_user = use_context::<ReadSignal<Option<SessionUser>>>().unwrap();
    let (institutions, set_institutions) = create_signal::<Vec<SamlInstitution>>(Vec::new());
    let (show_add_form, set_show_add_form) = signal(false);
    let (loading, set_loading) = signal(false);
    let (message, set_message) = create_signal::<Option<(String, bool)>>(None);

    // Form fields for adding new SAML config
    let (institution_name, set_institution_name) = signal("".to_string());
    let (entity_id, set_entity_id) = signal("".to_string());
    let (sso_url, set_sso_url) = signal("".to_string());
    let (slo_url, set_slo_url) = signal("".to_string());
    let (x509_cert, set_x509_cert) = signal("".to_string());
    let (metadata_url, set_metadata_url) = signal("".to_string());

    // Check if user has admin privileges
    let is_admin = move || {
        current_user
            .get()
            .map(|user| matches!(user.role, UserRole::Admin | UserRole::SuperAdmin))
            .unwrap_or(false)
    };

    // Load institutions on component mount
    Effect::new(move |_| {
        if is_admin() {
            spawn_local(async move {
                match get_saml_institutions().await {
                    Ok(institutions_list) => {
                        set_institutions.set(institutions_list);
                    }
                    Err(e) => {
                        log::info!("Failed to load SAML institutions: {:?}", e);
                        set_message
                            .set(Some((format!("Failed to load institutions: {}", e), false)));
                    }
                }
            });
        }
    });

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

                        // Clear form
                        set_institution_name.set("".to_string());
                        set_entity_id.set("".to_string());
                        set_sso_url.set("".to_string());
                        set_slo_url.set("".to_string());
                        set_x509_cert.set("".to_string());
                        set_metadata_url.set("".to_string());
                        set_show_add_form.set(false);

                        // Reload institutions
                        spawn_local(async move {
                            if let Ok(institutions_list) = get_saml_institutions().await {
                                set_institutions.set(institutions_list);
                            }
                        });
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

    view! {
        <div class="max-w-6xl mx-auto p-6">
            <div class="mb-6">
                <h1 class="text-3xl font-bold text-gray-900">"SAML Administration"</h1>
                <p class="mt-2 text-gray-600">"Manage SAML single sign-on configurations for institutions"</p>
            </div>

            {move || {
                if !is_admin() {
                    view! {
                        <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
                            "Access denied. Administrator privileges required."
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="space-y-6">
                            {move || {
                                message.get().map(|(msg, is_success)| {
                                    let bg_class = if is_success { "bg-green-100 border-green-400 text-green-700" } else { "bg-red-100 border-red-400 text-red-700" };
                                    view! {
                                        <div class={format!("border px-4 py-3 rounded {}", bg_class)}>
                                            {msg}
                                        </div>
                                    }
                                })
                            }}

                            // Header with Add button
                            <div class="flex justify-between items-center">
                                <h2 class="text-xl font-semibold text-gray-900">"SAML Institutions"</h2>
                                <button
                                    class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                                    on:click=move |_| set_show_add_form.update(|show| *show = !*show)
                                >
                                    {move || if show_add_form.get() { "Cancel" } else { "Add Institution" }}
                                </button>
                            </div>

                            // Add form
                            {move || {
                                if show_add_form.get() {
                                    view! {
                                        <div class="bg-gray-50 p-6 rounded-lg">
                                            <h3 class="text-lg font-medium text-gray-900 mb-4">"Add SAML Configuration"</h3>
                                            <form on:submit=move |ev| {
                                                ev.prevent_default();
                                                if !loading.get() {
                                                    handle_add_config.dispatch(());
                                                }
                                            }>
                                                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                                    <div>
                                                        <label class="block text-sm font-medium text-gray-700 mb-2">
                                                            "Institution Name" <span class="text-red-500">"*"</span>
                                                        </label>
                                                        <input
                                                            type="text"
                                                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                            prop:value=move || institution_name.get()
                                                            on:input=move |ev| set_institution_name.set(event_target_value(&ev))
                                                            prop:disabled=move || loading.get()
                                                            placeholder="University of Example"
                                                        />
                                                    </div>

                                                    <div>
                                                        <label class="block text-sm font-medium text-gray-700 mb-2">
                                                            "Entity ID" <span class="text-red-500">"*"</span>
                                                        </label>
                                                        <input
                                                            type="text"
                                                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                            prop:value=move || entity_id.get()
                                                            on:input=move |ev| set_entity_id.set(event_target_value(&ev))
                                                            prop:disabled=move || loading.get()
                                                            placeholder="https://idp.example.edu/saml/metadata"
                                                        />
                                                    </div>

                                                    <div>
                                                        <label class="block text-sm font-medium text-gray-700 mb-2">
                                                            "SLO URL (Optional)"
                                                        </label>
                                                        <input
                                                            type="url"
                                                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                            prop:value=move || slo_url.get()
                                                            on:input=move |ev| set_slo_url.set(event_target_value(&ev))
                                                            prop:disabled=move || loading.get()
                                                            placeholder="https://idp.example.edu/saml/slo"
                                                        />
                                                    </div>

                                                    <div>
                                                        <label class="block text-sm font-medium text-gray-700 mb-2">
                                                            "Metadata URL (Optional)"
                                                        </label>
                                                        <input
                                                            type="url"
                                                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                            prop:value=move || metadata_url.get()
                                                            on:input=move |ev| set_metadata_url.set(event_target_value(&ev))
                                                            prop:disabled=move || loading.get()
                                                            placeholder="https://idp.example.edu/saml/metadata"
                                                        />
                                                    </div>
                                                </div>

                                                <div class="mt-6">
                                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                                        "X.509 Certificate" <span class="text-red-500">"*"</span>
                                                    </label>
                                                    <textarea
                                                        rows="6"
                                                        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                                                        prop:value=move || x509_cert.get()
                                                        on:input=move |ev| set_x509_cert.set(event_target_value(&ev))
                                                        prop:disabled=move || loading.get()
                                                        placeholder="-----BEGIN CERTIFICATE-----
MIIEbzCCA1egAwIBAgIJAIYhQeZPzfH3MA0GCSqGSIb3DQEBCwUAMIGBMQswCQYD...
-----END CERTIFICATE-----"
                                                    ></textarea>
                                                    <p class="mt-1 text-sm text-gray-600">
                                                        "Paste the X.509 certificate from your identity provider"
                                                    </p>
                                                </div>

                                                <div class="mt-6 flex justify-end space-x-3">
                                                    <button
                                                        type="button"
                                                        class="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                                                        on:click=move |_| set_show_add_form.set(false)
                                                        prop:disabled=move || loading.get()
                                                    >
                                                        "Cancel"
                                                    </button>
                                                    <button
                                                        type="submit"
                                                        class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed"
                                                        prop:disabled=move || loading.get()
                                                    >
                                                        {move || {
                                                            if loading.get() {
                                                                "Creating..."
                                                            } else {
                                                                "Create Configuration"
                                                            }
                                                        }}
                                                    </button>
                                                </div>
                                            </form>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }
                            }}

                            // Institutions list
                            <div class="bg-white shadow rounded-lg">
                                <div class="px-6 py-4 border-b border-gray-200">
                                    <h3 class="text-lg font-medium text-gray-900">"Configured Institutions"</h3>
                                </div>
                                <div class="overflow-hidden">
                                    {move || {
                                        let institutions_list = institutions.get();
                                        if institutions_list.is_empty() {
                                            view! {
                                                <div class="p-6 text-center text-gray-500">
                                                    <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"/>
                                                    </svg>
                                                    <p class="mt-2">"No SAML institutions configured yet."</p>
                                                    <p class="text-sm">"Click 'Add Institution' to get started."</p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="divide-y divide-gray-200">
                                                    {institutions_list.into_iter().map(|institution| {
                                                        view! {
                                                            <div class="p-6 hover:bg-gray-50">
                                                                <div class="flex items-center justify-between">
                                                                    <div class="flex items-center">
                                                                        <div class="flex-shrink-0">
                                                                            <div class="h-10 w-10 bg-blue-100 rounded-lg flex items-center justify-center">
                                                                                <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"/>
                                                                                </svg>
                                                                            </div>
                                                                        </div>
                                                                        <div class="ml-4">
                                                                            <h4 class="text-lg font-medium text-gray-900">{institution.name.clone()}</h4>
                                                                            <p class="text-sm text-gray-500">
                                                                                "Institution ID: " {institution.id}
                                                                            </p>
                                                                        </div>
                                                                    </div>
                                                                    <div class="flex items-center space-x-2">
                                                                        {if institution.active {
                                                                            view! {
                                                                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                                                                                    "Active"
                                                                                </span>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! {
                                                                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                                                                                    "Inactive"
                                                                                </span>
                                                                            }.into_any()
                                                                        }}
                                                                        <button class="text-indigo-600 hover:text-indigo-900 text-sm font-medium">
                                                                            "Edit"
                                                                        </button>
                                                                        <button class="text-red-600 hover:text-red-900 text-sm font-medium">
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
                            </div>

                            // SAML Service Provider Info
                            <div class="bg-blue-50 border border-blue-200 rounded-lg p-6">
                                <h3 class="text-lg font-medium text-blue-900 mb-4">"Service Provider Information"</h3>
                                <p class="text-blue-800 mb-4">
                                    "Provide this information to your identity provider administrators:"
                                </p>
                                <div class="bg-white border border-blue-200 rounded p-4 font-mono text-sm">
                                    <div class="space-y-2">
                                        <div>
                                            <strong class="text-blue-900">"Entity ID:"</strong>
                                            <span class="ml-2 text-gray-700">{move || format!("{}/saml/metadata", std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()))}</span>
                                        </div>
                                        <div>
                                            <strong class="text-blue-900">"ACS URL:"</strong>
                                            <span class="ml-2 text-gray-700">{move || format!("{}/saml/acs", std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()))}</span>
                                        </div>
                                        <div>
                                            <strong class="text-blue-900">"SLS URL:"</strong>
                                            <span class="ml-2 text-gray-700">{move || format!("{}/saml/sls", std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()))}</span>
                                        </div>
                                        <div>
                                            <strong class="text-blue-900">"Metadata URL:"</strong>
                                            <span class="ml-2 text-gray-700">{move || format!("{}/saml/metadata", std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()))}</span>
                                        </div>
                                    </div>
                                </div>
                                <p class="mt-4 text-sm text-blue-700">
                                    "Note: Make sure your BASE_URL environment variable is set correctly for production use."
                                </p>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
