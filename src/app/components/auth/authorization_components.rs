use crate::app::models::user::UserJwt;
use crate::app::server_functions::auth::{get_current_user, login, logout, register};
use leptos::*;
use leptos_router::use_navigate;
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

    // Load the current user on component mount
    create_effect(move |_| {
        set_loading.set(true);
        logging::log!("AuthProvider: Loading current user");

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
    });

    // Add an effect to log whenever the current_user changes
    create_effect(move |_| {
        let user = current_user.get();
        logging::log!("AuthProvider: Current user updated: {:?}", user);
    });

    provide_context(current_user);
    provide_context(set_current_user);
    provide_context(loading);

    children()
}


#[component]
pub fn RequireAuth(children: Children) -> impl IntoView {
    let current_user = use_context::<ReadSignal<Option<UserJwt>>>().unwrap();
    let loading = use_context::<ReadSignal<bool>>().unwrap();

    let rendered_children = children();
    let navigate = use_navigate();

    create_effect(move |_| {
        if !loading.get() && current_user.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! {
        {move || {
            if loading.get() {
                view! { <div>"Loading..."</div> }
            } else if current_user.get().is_some() {
                view!{ <div>{rendered_children.clone()}</div>}
            } else {
                view! { <div></div> }
            }
        }}
    }
}

#[component]
pub fn RequireRole(
    #[prop(default = "user".to_string())] role: String,
    children: Children,
) -> impl IntoView {
    let current_user = use_context::<ReadSignal<Option<UserJwt>>>().unwrap();
    let loading = use_context::<ReadSignal<bool>>().unwrap();

    let navigate = use_navigate();

    // Render the children once and store the result
    let rendered_children = children();
    let role_mimic = role.clone();

    create_effect(move |_| {
        if !loading.get() {
            if let Some(user) = current_user.get() {
                match role.clone().as_str() {
                    "admin" => {
                        if !user.is_admin() {
                            navigate("/", Default::default());
                        }
                    }
                    "teacher" => {
                        if !user.is_teacher() {
                            navigate("/", Default::default());
                        }
                    }
                    _ => {}
                }
            } else {
                navigate("/login", Default::default());
            }
        }
    });

    view! {
        {move || {
            if loading.get() {
                view! { <div>"Loading..."</div> }
            } else if let Some(user) = current_user.get() {
                match role_mimic.as_str() {
                    "admin" => {
                        if user.is_admin() {
                            view!{ <div>{rendered_children.clone()}</div> }
                        } else {
                            view! { <div>"Unauthorized"</div> }
                        }
                    }
                    "teacher" => {
                        if user.is_teacher() {
                            view!{ <div>{rendered_children.clone()}</div> }
                        } else {
                            view! { <div>"Unauthorized"</div> }
                        }
                    }
                    _ => view!{ <div>{rendered_children.clone()}</div> }
                }
            } else {
                view! { <div></div> }
            }
        }}
    }
}

