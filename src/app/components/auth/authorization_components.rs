use crate::app::models::user::SessionUser;
use crate::app::server_functions::auth::get_current_user;
use leptos::*;
use leptos_router::{use_location, use_navigate};
use log::debug;

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let (current_user, set_current_user) = create_signal::<Option<SessionUser>>(None);
    let (loading, set_loading) = create_signal(true);
    let (initialized, set_initialized) = create_signal(false);
    let (redirect_after_auth, set_redirect_after_auth) = create_signal::<Option<String>>(None);

    // Use create_local_resource to avoid hydration issues
    let user_resource = create_local_resource(
        move || initialized.get(),
        move |_| async move {
            if initialized.get() {
                debug!("AuthProvider: Loading user");
                get_current_user().await
            } else {
                Ok(None)
            }
        },
    );

    // Initialize auth on client side only
    create_effect(move |_| {
        if !initialized.get_untracked() {
            set_initialized.set(true);

            // Store current path for post-login redirect
            let location = use_location();
            let current_path = location.pathname.get_untracked();
            if !is_auth_page(&current_path) {
                set_redirect_after_auth.set(Some(current_path));
            }
        }
    });

    // Track resource state
    create_effect(move |_| {
        match user_resource.get() {
            Some(Ok(user)) => {
                debug!("AuthProvider: User loaded: {:?}", user.is_some());
                set_current_user.set(user);
                set_loading.set(false);
            }
            Some(Err(err)) => {
                debug!("AuthProvider: Error loading user: {:?}", err);
                set_current_user.set(None);
                set_loading.set(false);
            }
            None => {
                // Still loading
                if initialized.get() {
                    set_loading.set(true);
                }
            }
        }
    });

    // Provide context
    provide_context(current_user);
    provide_context(set_current_user);
    provide_context(loading);
    provide_context(redirect_after_auth);
    provide_context(set_redirect_after_auth);

    children()
}

#[component]
pub fn RequireAuth(children: Children) -> impl IntoView {
    let current_user =
        use_context::<ReadSignal<Option<SessionUser>>>().expect("AuthProvider context not found");
    let loading = use_context::<ReadSignal<bool>>().expect("AuthProvider context not found");
    let set_redirect_after_auth =
        use_context::<WriteSignal<Option<String>>>().expect("AuthProvider context not found");

    let navigate = use_navigate();
    let location = use_location();
    let rendered_children = store_value(children());

    // Handle redirect when not authenticated - use create_effect with proper tracking
    create_effect(move |_| {
        let is_loading = loading.get();
        let user = current_user.get();

        debug!(
            "RequireAuth: loading={}, user={:?}",
            is_loading,
            user.is_some()
        );

        if !is_loading && user.is_none() {
            let current_path = location.pathname.get_untracked();
            if !is_auth_page(&current_path) {
                set_redirect_after_auth.set(Some(current_path));
            }
            debug!("RequireAuth: No user found, redirecting to login");
            navigate("/login", Default::default());
        }
    });

    // Use Suspense to handle loading states properly
    view! {
        <Suspense fallback=move || loading_view()>
            {move || {
                let is_loading = loading.get();
                let user = current_user.get();

                if is_loading {
                    debug!("RequireAuth: Showing loading view");
                    loading_view().into_view()
                } else if user.is_some() {
                    debug!("RequireAuth: User authenticated, showing content");
                    rendered_children.get_value().into_view()
                } else {
                    debug!("RequireAuth: No user, showing redirect view");
                    redirect_view().into_view()
                }
            }}
        </Suspense>
    }
}

#[component]
pub fn RequireRole(
    #[prop(default = "user".to_string())] role: String,
    children: Children,
) -> impl IntoView {
    let current_user =
        use_context::<ReadSignal<Option<SessionUser>>>().expect("AuthProvider context not found");
    let loading = use_context::<ReadSignal<bool>>().expect("AuthProvider context not found");
    let set_redirect_after_auth =
        use_context::<WriteSignal<Option<String>>>().expect("AuthProvider context not found");

    let navigate = use_navigate();
    let location = use_location();
    let rendered_children = store_value(children());
    let role_stored = store_value(role);

    create_effect(move |_| {
        let is_loading = loading.get();
        let user = current_user.get();
        let role = role_stored.get_value();

        debug!(
            "RequireRole: loading={}, user={:?}, required_role={}",
            is_loading,
            user.is_some(),
            role
        );

        if !is_loading {
            match user {
                Some(user) => {
                    if !user_has_role(&user, &role) {
                        debug!("RequireRole: User lacks required role: {}", role);
                        navigate("/", Default::default());
                    }
                }
                None => {
                    let current_path = location.pathname.get_untracked();
                    if !is_auth_page(&current_path) {
                        set_redirect_after_auth.set(Some(current_path));
                    }
                    debug!("RequireRole: No user found, redirecting to login");
                    navigate("/login", Default::default());
                }
            }
        }
    });

    view! {
        <Suspense fallback=move || loading_view()>
            {move || {
                let is_loading = loading.get();
                let user = current_user.get();
                let role = role_stored.get_value();

                if is_loading {
                    debug!("RequireRole: Showing loading view");
                    loading_view().into_view()
                } else if let Some(user) = user {
                    if user_has_role(&user, &role) {
                        debug!("RequireRole: User has required role, showing content");
                        rendered_children.get_value().into_view()
                    } else {
                        debug!("RequireRole: User lacks required role, showing unauthorized view");
                        unauthorized_view().into_view()
                    }
                } else {
                    debug!("RequireRole: No user, showing redirect view");
                    redirect_view().into_view()
                }
            }}
        </Suspense>
    }
}

#[component]
pub fn RequireAnyRole(roles: Vec<String>, children: Children) -> impl IntoView {
    let current_user =
        use_context::<ReadSignal<Option<SessionUser>>>().expect("AuthProvider context not found");
    let loading = use_context::<ReadSignal<bool>>().expect("AuthProvider context not found");
    let set_redirect_after_auth =
        use_context::<WriteSignal<Option<String>>>().expect("AuthProvider context not found");

    let navigate = use_navigate();
    let location = use_location();
    let rendered_children = store_value(children());
    let roles_stored = store_value(roles);

    create_effect(move |_| {
        let is_loading = loading.get();
        let user = current_user.get();
        let roles = roles_stored.get_value();

        if !is_loading {
            match user {
                Some(user) => {
                    if !user_has_any_role(&user, &roles) {
                        debug!("RequireAnyRole: User lacks required roles: {:?}", roles);
                        navigate("/", Default::default());
                    }
                }
                None => {
                    let current_path = location.pathname.get_untracked();
                    if !is_auth_page(&current_path) {
                        set_redirect_after_auth.set(Some(current_path));
                    }
                    debug!("RequireAnyRole: No user found, redirecting to login");
                    navigate("/login", Default::default());
                }
            }
        }
    });

    view! {
        <Suspense fallback=move || loading_view()>
            {move || {
                let is_loading = loading.get();
                let user = current_user.get();
                let roles = roles_stored.get_value();

                if is_loading {
                    loading_view().into_view()
                } else if let Some(user) = user {
                    if user_has_any_role(&user, &roles) {
                        rendered_children.get_value().into_view()
                    } else {
                        unauthorized_view().into_view()
                    }
                } else {
                    redirect_view().into_view()
                }
            }}
        </Suspense>
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

// Utility functions
fn is_auth_page(path: &str) -> bool {
    path.starts_with("/login")
        || path.starts_with("/register")
        || path.starts_with("/forgot-password")
        || path.starts_with("/reset-password")
}

fn user_has_role(user: &SessionUser, required_role: &str) -> bool {
    match required_role {
        "admin" => user.is_admin(),
        "teacher" => user.is_teacher(),
        "user" => user.is_user(),
        "guest" => user.is_guest(),
        _ => true,
    }
}

fn user_has_any_role(user: &SessionUser, required_roles: &[String]) -> bool {
    required_roles.iter().any(|role| user_has_role(user, role))
}

// Reusable view components
fn loading_view() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-screen">
            <div class="flex items-center space-x-2">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                <span class="text-lg text-gray-600">Loading...</span>
            </div>
        </div>
    }
}

fn redirect_view() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-screen">
            <div class="text-lg text-gray-600">Redirecting to login...</div>
        </div>
    }
}

fn unauthorized_view() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-screen">
            <div class="text-center">
                <h1 class="text-2xl font-bold text-red-600 mb-2">Unauthorized</h1>
                <p class="text-gray-600">"You don't have permission to access this page."</p>
                <a href="/" class="mt-4 inline-block text-blue-600 hover:underline">
                    Return to Dashboard
                </a>
            </div>
        </div>
    }
}

// Post-login redirect function
pub fn perform_post_login_redirect() {
    let redirect_after_auth =
        use_context::<ReadSignal<Option<String>>>().expect("AuthProvider context not found");
    let set_redirect_after_auth =
        use_context::<WriteSignal<Option<String>>>().expect("AuthProvider context not found");
    let navigate = use_navigate();

    if let Some(redirect_path) = redirect_after_auth.get_untracked() {
        set_redirect_after_auth.set(None);
        navigate(&redirect_path, Default::default());
    } else {
        navigate("/dashboard", Default::default());
    }
}

// Auth hooks for components that need user data
pub fn use_current_user() -> ReadSignal<Option<SessionUser>> {
    use_context::<ReadSignal<Option<SessionUser>>>().expect("AuthProvider context not found")
}

pub fn use_auth_loading() -> ReadSignal<bool> {
    use_context::<ReadSignal<bool>>().expect("AuthProvider context not found")
}

pub fn use_set_current_user() -> WriteSignal<Option<SessionUser>> {
    use_context::<WriteSignal<Option<SessionUser>>>().expect("AuthProvider context not found")
}
