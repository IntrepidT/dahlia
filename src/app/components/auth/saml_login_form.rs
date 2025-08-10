use crate::app::components::auth::authorization_components::perform_post_login_redirect;
use crate::app::models::user::SessionUser;
use crate::app::server_functions::auth::{get_current_user, login};
use crate::app::server_functions::saml_auth::{
    get_saml_institutions, initiate_saml_login, SamlInstitution,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

#[component]
pub fn SamlLoginForm() -> impl IntoView {
    let (username, set_username) = signal("".to_string());
    let (password, set_password) = signal("".to_string());
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (login_mode, set_login_mode) = signal("local"); // "local" or "saml"
    let (selected_institution, set_selected_institution) = create_signal::<Option<String>>(None);
    let (saml_institutions, set_saml_institutions) =
        create_signal::<Vec<SamlInstitution>>(Vec::new());
    let (loading, set_loading) = signal(false);

    let set_current_user = expect_context::<WriteSignal<Option<SessionUser>>>();
    let navigate = use_navigate();

    // Load SAML institutions on component mount
    Effect::new(move |_| {
        spawn_local(async move {
            match get_saml_institutions().await {
                Ok(institutions) => {
                    set_saml_institutions.set(institutions);
                }
                Err(e) => {
                    log::info!("Failed to load SAML institutions: {:?}", e);
                }
            }
        });
    });

    // Handle local login
    let handle_local_login = Action::new(move |_: &()| {
        let username = username.get();
        let password = password.get();

        async move {
            set_loading.set(true);
            set_error.set(None);

            if username.trim().is_empty() || password.trim().is_empty() {
                set_error.set(Some("Username and password are required".to_string()));
                set_loading.set(false);
                return;
            }

            match login(username, password).await {
                Ok(response) => {
                    if response.success {
                        set_current_user.set(response.user);
                        perform_post_login_redirect();
                    } else {
                        set_error.set(Some(response.message));
                    }
                }
                Err(err) => {
                    set_error.set(Some(format!("Login failed: {}", err)));
                }
            }

            set_loading.set(false);
        }
    });

    view! {
        <div class="max-w-md mx-auto mt-8 p-6 bg-white rounded-lg shadow-lg">
            <h2 class="text-2xl font-bold text-center mb-6 text-gray-800">"Login to Teapot Testing"</h2>

            {move || {
                error.get().map(|err| {
                    view! {
                        <div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded-md">
                            <div class="flex items-center">
                                <svg class="w-5 h-5 mr-2 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                                </svg>
                                <span class="text-sm">{err}</span>
                            </div>
                        </div>
                    }
                })
            }}

            // Login method selector
            <div class="mb-6">
                <div class="flex border rounded-lg overflow-hidden">
                    <button
                        class=move || {
                            let base = "flex-1 py-2 px-4 text-sm font-medium transition-colors";
                            if login_mode.get() == "local" {
                                format!("{} bg-blue-600 text-white", base)
                            } else {
                                format!("{} bg-gray-100 text-gray-700 hover:bg-gray-200", base)
                            }
                        }
                        on:click=move |_| set_login_mode.set("local")
                        disabled=move || loading.get()
                    >
                        "Username & Password"
                    </button>
                    <button
                        class=move || {
                            let base = "flex-1 py-2 px-4 text-sm font-medium transition-colors";
                            if login_mode.get() == "saml" {
                                format!("{} bg-blue-600 text-white", base)
                            } else {
                                format!("{} bg-gray-100 text-gray-700 hover:bg-gray-200", base)
                            }
                        }
                        on:click=move |_| set_login_mode.set("saml")
                        disabled=move || loading.get()
                    >
                        "Institution Login"
                    </button>
                </div>
            </div>

            {move || {
                if login_mode.get() == "local" {
                    view! {
                        <form on:submit=move |ev| {
                            ev.prevent_default();
                            if !loading.get() {
                                handle_local_login.dispatch(());
                            }
                        }>
                            <div class="mb-4">
                                <label class="block text-gray-700 text-sm font-medium mb-2" for="username">
                                    "Username"
                                </label>
                                <input
                                    id="username"
                                    type="text"
                                    class="w-full p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    prop:value=move || username.get()
                                    on:input=move |ev| set_username.set(event_target_value(&ev))
                                    prop:disabled=move || loading.get()
                                    placeholder="Enter your username"
                                />
                            </div>

                            <div class="mb-6">
                                <label class="block text-gray-700 text-sm font-medium mb-2" for="password">
                                    "Password"
                                </label>
                                <input
                                    id="password"
                                    type="password"
                                    class="w-full p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    prop:value=move || password.get()
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                    prop:disabled=move || loading.get()
                                    placeholder="Enter your password"
                                />
                            </div>

                            <button
                                type="submit"
                                class="w-full p-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors duration-200"
                                prop:disabled=move || loading.get()
                            >
                                {move || {
                                    if loading.get() {
                                        view! {
                                            <div class="flex items-center justify-center">
                                                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                                </svg>
                                                "Signing in..."
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            "Sign In"
                                        }.into_any()
                                    }
                                }}
                            </button>
                        </form>
                    }.into_any()
                } else {
                    // SAML Institution Login - Using button with JavaScript navigation
                    view! {
                        <div class="space-y-4">
                            {move || {
                                let institutions_list = saml_institutions.get();
                                if institutions_list.is_empty() {
                                    view! {
                                        <div class="p-4 bg-gray-100 rounded-md">
                                            <p class="text-sm text-gray-600 text-center">
                                                "No institutions configured for SAML login."
                                            </p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div>
                                            <label class="block text-gray-700 text-sm font-medium mb-2">
                                                "Select Your Institution"
                                            </label>
                                            <select
                                                class="w-full p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                on:change=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    if value.is_empty() {
                                                        set_selected_institution.set(None);
                                                    } else {
                                                        set_selected_institution.set(Some(value));
                                                    }
                                                }
                                                prop:disabled=move || loading.get()
                                            >
                                                <option value="">"-- Select Institution --"</option>
                                                {institutions_list.into_iter().map(|institution| {
                                                    let url_safe_name = institution.to_url_safe();
                                                    view! {
                                                        <option value={url_safe_name}>{institution.name}</option>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </select>

                                            // Use Leptos navigation for SPA routing with hydration guards
                                            {move || {
                                                if let Some(institution_id) = selected_institution.get() {
                                                    let login_url = format!("/saml/login?institution={}&relay_state=/dashboard", institution_id);

                                                    view! {
                                                        <a
                                                            href={login_url}
                                                            target="_blank"
                                                            rel="noopener noreferrer"
                                                            class="block w-full mt-4 p-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 text-center transition-colors duration-200 no-underline"
                                                        >
                                                            <div class="flex items-center justify-center">
                                                                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
                                                                </svg>
                                                                "Login with Institution"
                                                            </div>
                                                        </a>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <button
                                                            class="w-full mt-4 p-3 bg-gray-400 text-white rounded-md cursor-not-allowed"
                                                            disabled=true
                                                        >
                                                            "Select an institution first"
                                                        </button>
                                                    }.into_any()
                                                }
                                            }}

                                            <div class="mt-4 p-3 bg-blue-50 rounded-md">
                                                <p class="text-sm text-blue-700">
                                                    <strong>"Institution Login:"</strong>
                                                    " Your institution's credentials will be used to sign you in securely."
                                                </p>
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    }.into_any()
                }
            }}

            <div class="mt-6 pt-6 border-t border-gray-200">
                <p class="text-center text-sm text-gray-600">
                    "Need help? Contact your institution's IT support."
                </p>
            </div>
        </div>
    }
}
