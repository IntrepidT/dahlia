use crate::app::server_functions::authorization::{check_page_authorization, AuthorizationCheck};
use leptos::*;
use leptos_router::{use_location, use_navigate};
use log::debug;

#[component]
pub fn ServerAuthGuard(#[prop(into)] page_path: String, children: Children) -> impl IntoView {
    let navigate = use_navigate();
    let location = use_location();

    // Store children to avoid FnOnce issue
    let stored_children = store_value(children());

    // Create resource that checks authorization on the server
    let auth_check = create_resource(
        move || page_path.clone(),
        move |path| async move {
            debug!("ServerAuthGuard: Checking authorization for path: {}", path);
            check_page_authorization(path).await
        },
    );

    // Handle authorization result
    create_effect(move |_| {
        if let Some(Ok(auth_result)) = auth_check.get() {
            debug!("ServerAuthGuard: Auth result: {:?}", auth_result.authorized);

            if !auth_result.authorized {
                if let Some(redirect_url) = auth_result.redirect_url {
                    debug!("ServerAuthGuard: Redirecting to: {}", redirect_url);
                    navigate(&redirect_url, Default::default());
                }
            }
        }
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="flex items-center justify-center min-h-screen">
                <div class="flex items-center space-x-2">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    <span class="text-lg text-gray-600">Verifying permissions...</span>
                </div>
            </div>
        }>
            {move || {
                match auth_check.get() {
                    Some(Ok(auth_result)) => {
                        if auth_result.authorized {
                            debug!("ServerAuthGuard: User authorized, showing content");
                            stored_children.get_value().into_view()
                        } else {
                            debug!("ServerAuthGuard: User not authorized, showing unauthorized");
                            view! {
                                <div class="flex items-center justify-center min-h-screen">
                                    <div class="text-center">
                                        <h1 class="text-2xl font-bold text-red-600 mb-2">Unauthorized</h1>
                                        <p class="text-gray-600">"You don't have permission to access this page."</p>
                                        <a href="/" class="mt-4 inline-block text-blue-600 hover:underline">
                                            Return to Home
                                        </a>
                                    </div>
                                </div>
                            }.into_view()
                        }
                    }
                    Some(Err(_)) => {
                        view! {
                            <div class="flex items-center justify-center min-h-screen">
                                <div class="text-center">
                                    <h1 class="text-2xl font-bold text-red-600 mb-2">Error</h1>
                                    <p class="text-gray-600">"Unable to verify permissions. Please try again."</p>
                                    <a href="/login" class="mt-4 inline-block text-blue-600 hover:underline">
                                        Go to Login
                                    </a>
                                </div>
                            </div>
                        }.into_view()
                    }
                    None => {
                        // Still loading, handled by Suspense fallback
                        view! { <div></div> }.into_view()
                    }
                }
            }}
        </Suspense>
    }
}

#[component]
pub fn RequireServerAuth(
    #[prop(into, optional)] required_role: Option<String>,
    #[prop(into, optional)] required_roles: Option<Vec<String>>,
    children: Children,
) -> impl IntoView {
    let navigate = use_navigate();

    // Store children to avoid FnOnce issue
    let stored_children = store_value(children());

    // Create resource that checks specific role requirements
    let auth_check = create_resource(
        move || (required_role.clone(), required_roles.clone()),
        move |(role, roles)| async move {
            use crate::app::server_functions::authorization::check_authorization;
            debug!("RequireServerAuth: Checking role requirements");
            check_authorization(role, roles).await
        },
    );

    // Handle authorization result
    create_effect(move |_| {
        if let Some(Ok(auth_result)) = auth_check.get() {
            if !auth_result.authorized {
                if let Some(redirect_url) = auth_result.redirect_url {
                    debug!("RequireServerAuth: Redirecting to: {}", redirect_url);
                    navigate(&redirect_url, Default::default());
                }
            }
        }
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="flex items-center justify-center min-h-screen">
                <div class="flex items-center space-x-2">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    <span class="text-lg text-gray-600">Verifying permissions...</span>
                </div>
            </div>
        }>
            {move || {
                match auth_check.get() {
                    Some(Ok(auth_result)) => {
                        if auth_result.authorized {
                            stored_children.get_value().into_view()
                        } else {
                            view! {
                                <div class="flex items-center justify-center min-h-screen">
                                    <div class="text-center">
                                        <h1 class="text-2xl font-bold text-red-600 mb-2">Unauthorized</h1>
                                        <p class="text-gray-600">"You don't have permission to access this page."</p>
                                        <a href="/" class="mt-4 inline-block text-blue-600 hover:underline">
                                            Return to Home
                                        </a>
                                    </div>
                                </div>
                            }.into_view()
                        }
                    }
                    Some(Err(_)) => {
                        view! {
                            <div class="flex items-center justify-center min-h-screen">
                                <div class="text-center">
                                    <h1 class="text-2xl font-bold text-red-600 mb-2">Error</h1>
                                    <p class="text-gray-600">"Unable to verify permissions."</p>
                                </div>
                            </div>
                        }.into_view()
                    }
                    None => view! { <div></div> }.into_view()
                }
            }}
        </Suspense>
    }
}
