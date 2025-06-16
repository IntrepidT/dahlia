use crate::app::models::user::UserJwt;
use crate::app::server_functions::auth::{get_current_user, login, logout, register};
use leptos::*;
use leptos_router::{use_location, use_navigate};
use log::{debug, error, log};
use serde::Serialize;
#[cfg(feature = "ssr")]
use {
    lettre::transport::smtp::authentication::Credentials,
    lettre::{message::Message, SmtpTransport, Transport},
};

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let (current_user, set_current_user) = create_signal::<Option<UserJwt>>(None);
    let (loading, set_loading) = create_signal(true);
    let (initialized, set_initialized) = create_signal(false);
    let (redirect_after_auth, set_redirect_after_auth) = create_signal::<Option<String>>(None);

    // Only initialize auth on the client side
    create_effect(move |_| {
        // Only run this effect once, on the client side
        if !initialized.get() {
            set_initialized.set(true);

            // Store the current path if we're not already on login/register pages
            let location = use_location();
            let current_path = location.pathname.get();
            if !current_path.starts_with("/login") && !current_path.starts_with("/register") {
                set_redirect_after_auth.set(Some(current_path));
            }

            // Reduced delay and better error handling
            set_timeout(
                move || {
                    spawn_local(async move {
                        match get_current_user().await {
                            Ok(user) => {
                                logging::log!("AuthProvider: User loaded: {:?}", user);
                                set_current_user.set(user);
                            }
                            Err(err) => {
                                logging::log!("AuthProvider: Error loading user: {:?}", err);
                                set_current_user.set(None);
                            }
                        }
                        set_loading.set(false);
                    });
                },
                std::time::Duration::from_millis(50), // Reduced delay
            );
        }
    });

    provide_context(current_user);
    provide_context(set_current_user);
    provide_context(loading);
    provide_context(redirect_after_auth);
    provide_context(set_redirect_after_auth);

    children()
}

#[component]
pub fn RequireAuth(children: Children) -> impl IntoView {
    let current_user = use_context::<ReadSignal<Option<UserJwt>>>().unwrap();
    let loading = use_context::<ReadSignal<bool>>().unwrap();
    let redirect_after_auth = use_context::<ReadSignal<Option<String>>>().unwrap();
    let set_redirect_after_auth = use_context::<WriteSignal<Option<String>>>().unwrap();
    let navigate = use_navigate();
    let location = use_location();

    // Store children to avoid re-rendering
    let rendered_children = store_value(children());

    // Handle navigation with a separate effect - only redirect when loading is complete
    create_effect(move |_| {
        if !loading.get() && current_user.get().is_none() {
            let current_path = location.pathname.get();
            // Only store redirect path if we're not already on auth pages
            if !current_path.starts_with("/login") && !current_path.starts_with("/register") {
                set_redirect_after_auth.set(Some(current_path));
            }
            logging::log!("RequireAuth: No user found, redirecting to login");
            navigate("/login", Default::default());
        }
    });

    move || {
        if loading.get() {
            view! {
                <div class="flex items-center justify-center min-h-screen">
                    <div class="text-lg">Loading...</div>
                </div>
            }
            .into_view()
        } else if current_user.get().is_some() {
            rendered_children.get_value().into_view()
        } else {
            view! {
                <div class="flex items-center justify-center min-h-screen">
                    <div class="text-lg">Redirecting to login...</div>
                </div>
            }
            .into_view()
        }
    }
}

#[component]
pub fn RequireRole(
    #[prop(default = "user".to_string())] role: String,
    children: Children,
) -> impl IntoView {
    let current_user = use_context::<ReadSignal<Option<UserJwt>>>().unwrap();
    let loading = use_context::<ReadSignal<bool>>().unwrap();
    let redirect_after_auth = use_context::<ReadSignal<Option<String>>>().unwrap();
    let set_redirect_after_auth = use_context::<WriteSignal<Option<String>>>().unwrap();
    let navigate = use_navigate();
    let location = use_location();

    // Store children and role to avoid re-rendering
    let rendered_children = store_value(children());
    let role_stored = store_value(role);

    create_effect(move |_| {
        if !loading.get() {
            let role = role_stored.get_value();
            match current_user.get() {
                Some(user) => {
                    let has_permission = match role.as_str() {
                        "admin" => user.is_admin(),
                        "teacher" => user.is_teacher(),
                        _ => true,
                    };

                    if !has_permission {
                        logging::log!("RequireRole: User lacks required role: {}", role);
                        navigate("/", Default::default());
                    }
                }
                None => {
                    let current_path = location.pathname.get();
                    if !current_path.starts_with("/login") && !current_path.starts_with("/register")
                    {
                        set_redirect_after_auth.set(Some(current_path));
                    }
                    logging::log!("RequireRole: No user found, redirecting to login");
                    navigate("/login", Default::default());
                }
            }
        }
    });

    move || {
        let role = role_stored.get_value();
        if loading.get() {
            view! {
                <div class="flex items-center justify-center min-h-screen">
                    <div class="text-lg">Loading...</div>
                </div>
            }
            .into_view()
        } else if let Some(user) = current_user.get() {
            let has_permission = match role.as_str() {
                "admin" => user.is_admin(),
                "teacher" => user.is_teacher(),
                _ => true,
            };

            if has_permission {
                rendered_children.get_value().into_view()
            } else {
                view! {
                    <div class="flex items-center justify-center min-h-screen">
                        <div class="text-lg text-red-600">Unauthorized - Insufficient permissions</div>
                    </div>
                }.into_view()
            }
        } else {
            view! {
                <div class="flex items-center justify-center min-h-screen">
                    <div class="text-lg">Redirecting to login...</div>
                </div>
            }
            .into_view()
        }
    }
}

#[component]
pub fn RequireAnyRole(roles: Vec<String>, children: Children) -> impl IntoView {
    let current_user = use_context::<ReadSignal<Option<UserJwt>>>().unwrap();
    let loading = use_context::<ReadSignal<bool>>().unwrap();
    let redirect_after_auth = use_context::<ReadSignal<Option<String>>>().unwrap();
    let set_redirect_after_auth = use_context::<WriteSignal<Option<String>>>().unwrap();
    let navigate = use_navigate();
    let location = use_location();

    // Store values to avoid re-rendering
    let rendered_children = store_value(children());
    let roles_stored = store_value(roles);

    create_effect(move |_| {
        if !loading.get() {
            let roles = roles_stored.get_value();
            match current_user.get() {
                Some(user) => {
                    let has_permission = roles.iter().any(|role| match role.as_str() {
                        "admin" => user.is_admin(),
                        "teacher" => user.is_teacher(),
                        _ => true,
                    });

                    if !has_permission {
                        logging::log!("RequireAnyRole: User lacks required roles: {:?}", roles);
                        navigate("/", Default::default());
                    }
                }
                None => {
                    let current_path = location.pathname.get();
                    if !current_path.starts_with("/login") && !current_path.starts_with("/register")
                    {
                        set_redirect_after_auth.set(Some(current_path));
                    }
                    logging::log!("RequireAnyRole: No user found, redirecting to login");
                    navigate("/login", Default::default());
                }
            }
        }
    });

    move || {
        let roles = roles_stored.get_value();
        if loading.get() {
            view! {
                <div class="flex items-center justify-center min-h-screen">
                    <div class="text-lg">Loading...</div>
                </div>
            }
            .into_view()
        } else if let Some(user) = current_user.get() {
            let has_permission = roles.iter().any(|role| match role.as_str() {
                "admin" => user.is_admin(),
                "teacher" => user.is_teacher(),
                _ => true,
            });

            if has_permission {
                rendered_children.get_value().into_view()
            } else {
                view! {
                    <div class="flex items-center justify-center min-h-screen">
                        <div class="text-lg text-red-600">Unauthorized - Insufficient permissions</div>
                    </div>
                }.into_view()
            }
        } else {
            view! {
                <div class="flex items-center justify-center min-h-screen">
                    <div class="text-lg">Redirecting to login...</div>
                </div>
            }
            .into_view()
        }
    }
}

#[component]
pub fn RequireAdminOrTeacher(children: Children) -> impl IntoView {
    view! {
        <RequireAnyRole roles=vec!["admin".to_string(), "teacher".to_string()]>
            {children()}
        </RequireAnyRole>
    }
}

// Updated hook with better redirect handling
pub fn use_auth_redirect() -> (ReadSignal<Option<UserJwt>>, Signal<bool>) {
    let user = use_context::<ReadSignal<Option<UserJwt>>>().expect("AuthProvider not found");
    let loading = use_context::<Signal<bool>>().unwrap_or_else(|| Signal::derive(|| false));
    let redirect_after_auth = use_context::<ReadSignal<Option<String>>>().unwrap();
    let set_redirect_after_auth = use_context::<WriteSignal<Option<String>>>().unwrap();
    let navigate = use_navigate();
    let location = use_location();

    // Handle redirect for unauthenticated users - only after loading completes
    create_effect(move |_| {
        if !loading.get() && user.get().is_none() {
            let current_path = location.pathname.get();
            if !current_path.starts_with("/login") && !current_path.starts_with("/register") {
                set_redirect_after_auth.set(Some(current_path));
            }
            logging::log!("use_auth_redirect: No user found, redirecting to login");
            navigate("/login", Default::default());
        }
    });

    (user, loading)
}

pub fn use_role_redirect(
    required_roles: Vec<String>,
) -> (ReadSignal<Option<UserJwt>>, Signal<bool>) {
    let user = use_context::<ReadSignal<Option<UserJwt>>>().expect("AuthProvider not found");
    let loading = use_context::<Signal<bool>>().unwrap_or_else(|| Signal::derive(|| false));
    let redirect_after_auth = use_context::<ReadSignal<Option<String>>>().unwrap();
    let set_redirect_after_auth = use_context::<WriteSignal<Option<String>>>().unwrap();
    let navigate = use_navigate();
    let location = use_location();

    // Handle redirect for unauthorized users - only after loading completes
    create_effect(move |_| {
        if !loading.get() {
            match user.get() {
                Some(user_data) => {
                    let has_permission = required_roles.iter().any(|role| match role.as_str() {
                        "admin" => user_data.is_admin(),
                        "teacher" => user_data.is_teacher(),
                        _ => true,
                    });

                    if !has_permission {
                        logging::log!(
                            "use_role_redirect: User lacks required roles: {:?}",
                            required_roles
                        );
                        navigate("/", Default::default());
                    }
                }
                None => {
                    let current_path = location.pathname.get();
                    if !current_path.starts_with("/login") && !current_path.starts_with("/register")
                    {
                        set_redirect_after_auth.set(Some(current_path));
                    }
                    logging::log!("use_role_redirect: No user found, redirecting to login");
                    navigate("/login", Default::default());
                }
            }
        }
    });

    (user, loading)
}

pub fn perform_post_login_redirect() {
    let redirect_after_auth = use_context::<ReadSignal<Option<String>>>().unwrap();
    let set_redirect_after_auth = use_context::<WriteSignal<Option<String>>>().unwrap();
    let navigate = use_navigate();

    if let Some(redirect_path) = redirect_after_auth.get() {
        set_redirect_after_auth.set(None);
        navigate(&redirect_path, Default::default());
    } else {
        navigate("/dashboard", Default::default());
    }
}
