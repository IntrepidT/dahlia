use crate::app::components::auth::server_auth_components::ServerAuthGuard;
use crate::app::components::dashboard::scores_ledger::ScoresLedger;
use crate::app::components::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::models::user::{SessionUser, UserRole};
use crate::app::server_functions::saml_auth::{create_saml_config, get_saml_institutions};
use leptos::*;
use leptos_router::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <ServerAuthGuard page_path="/dashboard">
            <DashboardContent />
        </ServerAuthGuard>
    }
}

#[component]
fn DashboardContent() -> impl IntoView {
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::Overview);
    let location = use_location();

    create_effect(move |_| {
        let path = location.pathname.get();
        if path.starts_with("/dashboard") {
            set_selected_view(SidebarSelected::Overview);
        } else if path.starts_with("/studentview") {
            set_selected_view(SidebarSelected::StudentView);
        } else if path.starts_with("/teachers") {
            set_selected_view(SidebarSelected::TeacherView);
        } else if path.starts_with("/testsessions") {
            set_selected_view(SidebarSelected::Live);
        }
    });

    view! {
        <div class="bg-[#F9F9F8] h-full">
            <Header />
            <div class="flex h-full">
                <DashboardSidebar
                    selected_item=selected_view
                    set_selected_item=set_selected_view
                />
                <main class="flex-1 mt-16 ml-20 px-10">
                    {move || match selected_view() {
                        SidebarSelected::Overview => view! {
                            <div>
                                <div class="flex justify-between items-center mb-4">
                                    <div class="text-2xl font-bold text-[#2E3A59]">
                                        "Overview"
                                    </div>
                                    <SamlTestButton />
                                </div>
                                <div class="flex gap-4 w-full">
                                    <div class="flex-1 w-1/2">
                                        <div class="shadow-lg border-gray border-2 h-[20rem] rounded-lg">
                                            <h1 class="text-base font-bold text-xl ml-2 p-2 text-[#2E3A59]">
                                                Today
                                            </h1>
                                            <hr class="text-sm text-gray" />
                                        </div>
                                    </div>
                                    <div class="flex-1 w-1/2">
                                        <div class="shadow-lg border-gray border-2 h-[20rem] rounded-lg">
                                            <h1 class="text-base font-bold text-xl ml-2 p-2 text-[#2E3A59]">
                                                Logs
                                            </h1>
                                            <hr class="text-sm text-gray" />
                                        </div>
                                    </div>
                                </div>
                                <div class="text-2xl font-bold mt-5 ">
                                    <div class="flex-1 w-full h-[20rem] rounded-lg mt-2">
                                        <Suspense fallback=move || view! {
                                            <div class="flex justify-center items-center h-40">
                                                <svg class="animate-spin h-6 w-6 text-indigo-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                                </svg>
                                                <span class="ml-2 text-[#2E3A59]">Loading scores...</span>
                                            </div>
                                        }>
                                            <ScoresLedger />
                                        </Suspense>
                                    </div>
                                </div>
                            </div>
                        },
                        _ => view! {
                            <div class="text-2xl font-bold text-[#2E3A59]">
                                "Admin-only content"
                            </div>
                        }
                    }}
                </main>
            </div>
        </div>
    }
}

// ONLY ONE SamlTestButton component - remove the duplicate!
#[component]
fn SamlTestButton() -> impl IntoView {
    let current_user = use_context::<ReadSignal<Option<SessionUser>>>().unwrap();
    let (loading, set_loading) = create_signal(false);
    let (message, set_message) = create_signal::<Option<(String, bool)>>(None);
    let (show_test_form, set_show_test_form) = create_signal(false);
    let (test_step, set_test_step) = create_signal(0); // Track progress

    // Check if user has admin privileges
    let is_admin = move || {
        current_user
            .get()
            .map(|user| matches!(user.role, UserRole::Admin | UserRole::SuperAdmin))
            .unwrap_or(false)
    };

    let create_test_saml = create_action(move |_: &()| {
        async move {
            set_loading.set(true);
            set_message.set(None);
            set_test_step.set(1);

            // Create a test SAML configuration using Mock SAML
            let test_config = create_saml_config(
                "Mock SAML Test".to_string(),
                "https://mocksaml.com/api/saml/metadata".to_string(),
                "https://mocksaml.com/api/saml/sso".to_string(),
                Some("https://mocksaml.com/api/saml/slo".to_string()),
                // Mock SAML's actual certificate
                "-----BEGIN CERTIFICATE-----
MIIC4jCCAcoCCQC33wnybT5QZDANBgkqhkiG9w0BAQsFADAyMQswCQYDVQQGEwJV
SzEPMA0GA1UECgwGQm94eUhRMRIwEAYDVQQDDAlNb2NrIFNBTUwwIBcNMjIwMjI4
MjE0NjM4WhgPMzAyMTA3MDEyMTQ2MzhaMDIxCzAJBgNVBAYTAlVLMQ8wDQYDVQQK
DAZCb3h5SFExEjAQBgNVBAMMCU1vY2sgU0FNTDCCASIwDQYJKoZIhvcNAQEBBQAD
ggEPADCCAQoCggEBALGfYettMsct1T6tVUwTudNJH5Pnb9GGnkXi9Zw/e6x45DD0
RuRONbFlJ2T4RjAE/uG+AjXxXQ8o2SZfb9+GgmCHuTJFNgHoZ1nFVXCmb/Hg8Hpd
4vOAGXndixaReOiq3EH5XvpMjMkJ3+8+9VYMzMZOjkgQtAqO36eAFFfNKX7dTj3V
pwLkvz6/KFCq8OAwY+AUi4eZm5J57D31GzjHwfjH9WTeX0MyndmnNB1qV75qQR3b
2/W5sGHRv+9AarggJkF+ptUkXoLtVA51wcfYm6hILptpde5FQC8RWY1YrswBWAEZ
NfyrR4JeSweElNHg4NVOs4TwGjOPwWGqzTfgTlECAwEAATANBgkqhkiG9w0BAQsF
AAOCAQEAAYRlYflSXAWoZpFfwNiCQVE5d9zZ0DPzNdWhAybXcTyMf0z5mDf6FWBW
5Gyoi9u3EMEDnzLcJNkwJAAc39Apa4I2/tml+Jy29dk8bTyX6m93ngmCgdLh5Za4
khuU3AM3L63g7VexCuO7kwkjh/+LqdcIXsVGO6XDfu2QOs1Xpe9zIzLpwm/RNYeX
UjbSj5ce/jekpAw7qyVVL4xOyh8AtUW1ek3wIw1MJvEgEPt0d16oshWJpoS1OT8L
r/22SvYEo3EmSGdTVGgk3x3s+A0qWAqTcyjr7Q4s/GKYRFfomGwz0TZ4Iw1ZN99M
m0eo2USlSRTVl7QHRTuiuSThHpLKQQ==
-----END CERTIFICATE-----"
                    .to_string(),
                Some("https://mocksaml.com/api/saml/metadata".to_string()),
            )
            .await;

            match test_config {
                Ok(response) => {
                    if response.success {
                        set_message.set(Some((
                            "✅ Mock SAML configuration created successfully!".to_string(),
                            true,
                        )));
                        set_test_step.set(2);

                        // Now test if we can retrieve it
                        spawn_local(async move {
                            match get_saml_institutions().await {
                                Ok(institutions) => {
                                    let test_institution = institutions
                                        .iter()
                                        .find(|inst| inst.name == "Mock SAML Test");
                                    if test_institution.is_some() {
                                        set_message.set(Some(("🎉 SAML configuration test PASSED! Ready for login testing.".to_string(), true)));
                                        set_test_step.set(3);
                                    } else {
                                        set_message.set(Some((
                                            "⚠️ Configuration created but not found in retrieval."
                                                .to_string(),
                                            false,
                                        )));
                                    }
                                }
                                Err(e) => {
                                    set_message.set(Some((
                                        format!("❌ Failed to retrieve institutions: {}", e),
                                        false,
                                    )));
                                }
                            }
                        });
                    } else {
                        set_message.set(Some((format!("❌ {}", response.message), false)));
                    }
                }
                Err(e) => {
                    set_message.set(Some((
                        format!("❌ Failed to create SAML config: {}", e),
                        false,
                    )));
                }
            }

            set_loading.set(false);
        }
    });

    view! {
        {move || {
            if is_admin() {
                view! {
                    <div class="flex flex-col items-end space-y-2">
                        <div class="flex space-x-2">
                            <button
                                class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed text-sm"
                                on:click=move |_| set_show_test_form.update(|show| *show = !*show)
                            >
                                "🧪 Test SAML End-to-End"
                            </button>
                        </div>

                        {move || {
                            if show_test_form.get() {
                                let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

                                view! {
                                    <div class="absolute top-16 right-0 z-10 bg-white border border-gray-300 rounded-lg shadow-lg p-4 w-96">
                                        <div class="flex justify-between items-center mb-3">
                                            <h3 class="text-lg font-medium text-gray-900">"SAML End-to-End Test"</h3>
                                            <button
                                                class="text-gray-400 hover:text-gray-600"
                                                on:click=move |_| {
                                                    set_show_test_form.set(false);
                                                    set_test_step.set(0);
                                                    set_message.set(None);
                                                }
                                            >
                                                "✕"
                                            </button>
                                        </div>

                                        {move || {
                                            message.get().map(|(msg, is_success)| {
                                                let bg_class = if is_success { "bg-green-50 border-green-200 text-green-800" } else { "bg-red-50 border-red-200 text-red-800" };
                                                view! {
                                                    <div class={format!("border px-3 py-2 rounded mb-3 text-sm {}", bg_class)}>
                                                        {msg}
                                                    </div>
                                                }
                                            })
                                        }}

                                        <div class="space-y-4">
                                            // Step 1: Create SAML Config
                                            <div class={format!("p-3 rounded border {}",
                                                if test_step.get() >= 1 { "bg-green-50 border-green-200" }
                                                else { "bg-gray-50 border-gray-200" }
                                            )}>
                                                <div class="flex items-center justify-between mb-2">
                                                    <h4 class="font-medium text-gray-900">"Step 1: Create SAML Config"</h4>
                                                    {move || if test_step.get() >= 2 {
                                                        view! { <span class="text-green-600">"✅"</span> }
                                                    } else {
                                                        view! { <span></span> }
                                                    }}
                                                </div>
                                                <button
                                                    class="w-full px-3 py-2 bg-blue-600 text-white rounded text-sm hover:bg-blue-700 disabled:bg-gray-400"
                                                    on:click=move |_| {
                                                        if !loading.get() {
                                                            create_test_saml.dispatch(());
                                                        }
                                                    }
                                                    prop:disabled=move || loading.get() || (test_step.get() >= 2)
                                                >
                                                    {move || {
                                                        if loading.get() {
                                                            "Creating..."
                                                        } else if test_step.get() >= 2 {
                                                            "✅ Config Created"
                                                        } else {
                                                            "Create Mock SAML Config"
                                                        }
                                                    }}
                                                </button>
                                            </div>

                                            // Step 2: Configure Mock SAML
                                            <div class={format!("p-3 rounded border {}",
                                                if test_step.get() >= 2 { "bg-blue-50 border-blue-200" }
                                                else { "bg-gray-50 border-gray-200" }
                                            )}>
                                                <h4 class="font-medium text-gray-900 mb-2">"Step 2: Configure Mock SAML"</h4>
                                                <p class="text-sm text-gray-600 mb-3">
                                                    "Configure Mock SAML to point back to your service:"
                                                </p>

                                                <div class="space-y-2 text-xs bg-white p-3 rounded border">
                                                    <div>
                                                        <strong>"1. Visit: "</strong>
                                                        <a href="https://mocksaml.com/" target="_blank" class="text-blue-600 hover:underline">
                                                            "https://mocksaml.com/"
                                                        </a>
                                                    </div>
                                                    <div>
                                                        <strong>"2. Enter your SP Metadata URL:"</strong>
                                                        <div class="mt-1 p-2 bg-gray-100 rounded font-mono break-all">
                                                            {format!("{}/saml/metadata", base_url)}
                                                        </div>
                                                    </div>
                                                    <div>
                                                        <strong>"3. Or manually configure:"</strong>
                                                        <div class="ml-2 space-y-1">
                                                            <div>
                                                                "ACS URL: "
                                                                <code class="bg-gray-100 px-1 rounded text-xs">
                                                                    {format!("{}/saml/acs", base_url)}
                                                                </code>
                                                            </div>
                                                            <div>
                                                                "Entity ID: "
                                                                <code class="bg-gray-100 px-1 rounded text-xs">
                                                                    {format!("{}/saml/metadata", base_url)}
                                                                </code>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>

                                            // Step 3: Test Login
                                            <div class={format!("p-3 rounded border {}",
                                                if test_step.get() >= 3 { "bg-green-50 border-green-200" }
                                                else { "bg-gray-50 border-gray-200" }
                                            )}>
                                                <h4 class="font-medium text-gray-900 mb-2">"Step 3: Test SAML Login"</h4>

                                                {move || if test_step.get() >= 3 {
                                                    // Use the correct institution name format (URL-safe)
                                                    let login_url = format!("{}/saml/login?institution=mock-saml-test", base_url);
                                                    view! {
                                                        <div class="space-y-2">
                                                            <p class="text-sm text-gray-600">
                                                                "After configuring Mock SAML, test the login flow:"
                                                            </p>
                                                            <a
                                                                href=login_url
                                                                target="_blank"
                                                                rel="noopener noreferrer"
                                                                class="inline-block w-full px-3 py-2 bg-green-600 text-white rounded text-sm hover:bg-green-700 transition-colors text-center"
                                                            >
                                                                "🔗 Test SAML Login Flow"
                                                            </a>
                                                            <p class="text-xs text-gray-500">
                                                                "This will redirect to Mock SAML, then back to your app"
                                                            </p>
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! {
                                                        <p class="text-sm text-gray-500">
                                                            "Complete steps 1 and 2 first"
                                                        </p>
                                                    }.into_view()
                                                }}
                                            </div>

                                            // Additional helpful info
                                            <div class="p-3 bg-yellow-50 border border-yellow-200 rounded">
                                                <h4 class="font-medium text-yellow-800 mb-1">"💡 Troubleshooting"</h4>
                                                <ul class="text-xs text-yellow-700 space-y-1">
                                                    <li>"• Check browser console for errors"</li>
                                                    <li>"• Verify your BASE_URL environment variable"</li>
                                                    <li>"• Make sure your app is accessible from the internet for Mock SAML"</li>
                                                    <li>
                                                        "• Check SAML response at: "
                                                        <code class="bg-yellow-100 px-1 rounded">"/saml/health"</code>
                                                    </li>
                                                </ul>
                                            </div>
                                        </div>
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }
                        }}
                    </div>
                }.into_view()
            } else {
                view! { <div></div> }.into_view()
            }
        }}
    }
}
