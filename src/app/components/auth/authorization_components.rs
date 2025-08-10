use crate::app::models::user::SessionUser;
use crate::app::server_functions::auth::get_current_user;
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};
use log::debug;

#[component]
pub fn AuthProvider(children: ChildrenFn) -> impl IntoView {
    let (current_user, set_current_user) = signal::<Option<SessionUser>>(None);
    let (loading, set_loading) = signal(true);
    let (initialized, set_initialized) = signal(false);
    let (redirect_after_auth, set_redirect_after_auth) = signal::<Option<String>>(None);

    let user_resource = Resource::new(
        move || initialized.get(),
        move |is_initialized| async move {
            if is_initialized {
                debug!("AuthProvider: Loading user");
                get_current_user().await
            } else {
                Ok(None)
            }
        },
    );

    // Initialize auth on client side only
    Effect::new(move |_| {
        if !initialized.get_untracked() {
            set_initialized.set(true);

            // Store current path for post-login redirect - use get_untracked to avoid tracking
            let location = use_location();
            let current_path = location.pathname.get_untracked();
            if !is_auth_page(&current_path) {
                set_redirect_after_auth.set(Some(current_path));
            }
        }
    });

    // Track resource state
    Effect::new(move |_| {
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
pub fn RequireAuth(children: ChildrenFn) -> impl IntoView {
    let current_user = expect_context::<ReadSignal<Option<SessionUser>>>();
    let loading = expect_context::<ReadSignal<bool>>();
    let set_redirect_after_auth = expect_context::<WriteSignal<Option<String>>>();

    let navigate = use_navigate();
    let location = use_location();

    // Handle redirect when not authenticated
    Effect::new(move |_| {
        let is_loading = loading.get();
        let user = current_user.get();

        debug!(
            "RequireAuth: loading={}, user={:?}",
            is_loading,
            user.is_some()
        );

        if !is_loading && user.is_none() {
            // Use get_untracked to avoid creating unnecessary reactive dependencies
            let current_path = location.pathname.get_untracked();
            if !is_auth_page(&current_path) {
                set_redirect_after_auth.set(Some(current_path));
            }
            debug!("RequireAuth: No user found, redirecting to login");
            navigate("/login", Default::default());
        }
    });

    // Use Show component with proper reactive tracking
    view! {
        <Show
            when=move || !loading.get() && current_user.get().is_some()
            fallback=move || {
                if loading.get() {
                    loading_view().into_any()
                } else {
                    redirect_view().into_any()
                }
            }
        >
            {children()}
        </Show>
    }
}

#[component]
pub fn RequireRole(
    #[prop(default = "user".to_string())] role: String,
    children: ChildrenFn,
) -> impl IntoView {
    let current_user = expect_context::<ReadSignal<Option<SessionUser>>>();
    let loading = expect_context::<ReadSignal<bool>>();
    let set_redirect_after_auth = expect_context::<WriteSignal<Option<String>>>();

    let navigate = use_navigate();
    let location = use_location();

    // Clone role for use in both closures
    let role_for_effect = role.clone();
    let role_for_when = role.clone();

    Effect::new(move |_| {
        let is_loading = loading.get();
        let user = current_user.get();

        debug!(
            "RequireRole: loading={}, user={:?}, required_role={}",
            is_loading,
            user.is_some(),
            role_for_effect
        );

        if !is_loading {
            match user {
                Some(user) => {
                    if !user_has_role(&user, &role_for_effect) {
                        debug!("RequireRole: User lacks required role: {}", role_for_effect);
                        navigate("/", Default::default());
                    }
                }
                None => {
                    // Use get_untracked to avoid creating unnecessary reactive dependencies
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

    // Use Show component with proper reactive tracking
    view! {
        <Show
            when=move || {
                if loading.get() {
                    false
                } else if let Some(user) = current_user.get() {
                    user_has_role(&user, &role_for_when)
                } else {
                    false
                }
            }
            fallback=move || {
                if loading.get() {
                    loading_view().into_any()
                } else if current_user.get().is_some() {
                    unauthorized_view().into_any()
                } else {
                    redirect_view().into_any()
                }
            }
        >
            {children()}
        </Show>
    }
}

#[component]
pub fn RequireAnyRole(roles: Vec<String>, children: ChildrenFn) -> impl IntoView {
    let current_user = expect_context::<ReadSignal<Option<SessionUser>>>();
    let loading = expect_context::<ReadSignal<bool>>();
    let set_redirect_after_auth = expect_context::<WriteSignal<Option<String>>>();

    let navigate = use_navigate();
    let location = use_location();

    // Clone roles for use in both closures
    let roles_for_effect = roles.clone();
    let roles_for_when = roles.clone();

    Effect::new(move |_| {
        let is_loading = loading.get();
        let user = current_user.get();

        if !is_loading {
            match user {
                Some(user) => {
                    if !user_has_any_role(&user, &roles_for_effect) {
                        debug!(
                            "RequireAnyRole: User lacks required roles: {:?}",
                            roles_for_effect
                        );
                        navigate("/", Default::default());
                    }
                }
                None => {
                    // Use get_untracked to avoid creating unnecessary reactive dependencies
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

    // Use Show component with proper reactive tracking
    view! {
        <Show
            when=move || {
                if loading.get() {
                    false
                } else if let Some(user) = current_user.get() {
                    user_has_any_role(&user, &roles_for_when)
                } else {
                    false
                }
            }
            fallback=move || {
                if loading.get() {
                    loading_view().into_any()
                } else if current_user.get().is_some() {
                    unauthorized_view().into_any()
                } else {
                    redirect_view().into_any()
                }
            }
        >
            {children()}
        </Show>
    }
}

#[component]
pub fn RequireAdminOrTeacher(children: ChildrenFn) -> impl IntoView {
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
    let redirect_after_auth = expect_context::<ReadSignal<Option<String>>>();
    let set_redirect_after_auth = expect_context::<WriteSignal<Option<String>>>();
    let navigate = use_navigate();

    // Use get_untracked for one-time read that doesn't need tracking
    if let Some(redirect_path) = redirect_after_auth.get_untracked() {
        set_redirect_after_auth.set(None);
        navigate(&redirect_path, Default::default());
    } else {
        navigate("/dashboard", Default::default());
    }
}

// Auth hooks for components that need user data
pub fn use_current_user() -> ReadSignal<Option<SessionUser>> {
    expect_context::<ReadSignal<Option<SessionUser>>>()
}

pub fn use_auth_loading() -> ReadSignal<bool> {
    expect_context::<ReadSignal<bool>>()
}

pub fn use_set_current_user() -> WriteSignal<Option<SessionUser>> {
    expect_context::<WriteSignal<Option<SessionUser>>>()
}
