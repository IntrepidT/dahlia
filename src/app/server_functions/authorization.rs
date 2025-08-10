use crate::app::models::user::{SessionUser, UserRole};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorizationCheck {
    pub authorized: bool,
    pub user: Option<SessionUser>,
    pub required_role: Option<String>,
    pub redirect_url: Option<String>,
}

#[server]
pub async fn check_authorization(
    required_role: Option<String>,
    required_roles: Option<Vec<String>>,
) -> Result<AuthorizationCheck, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::server_functions::auth::get_current_user;

        // Get current user from session
        let current_user = get_current_user().await?;

        match current_user {
            Some(user) => {
                let authorized = if let Some(role) = required_role.as_ref() {
                    // Check single role
                    match role.as_str() {
                        "admin" => user.is_admin(),
                        "teacher" => user.is_teacher(),
                        "user" => user.is_user(),
                        "guest" => user.is_guest(),
                        _ => false,
                    }
                } else if let Some(roles) = required_roles.as_ref() {
                    // Check multiple roles (any match)
                    roles.iter().any(|role| match role.as_str() {
                        "admin" => user.is_admin(),
                        "teacher" => user.is_teacher(),
                        "user" => user.is_user(),
                        "guest" => user.is_guest(),
                        _ => false,
                    })
                } else {
                    // No role required, just need to be authenticated
                    true
                };

                Ok(AuthorizationCheck {
                    authorized,
                    user: Some(user),
                    required_role,
                    redirect_url: if authorized {
                        None
                    } else {
                        Some("/".to_string())
                    },
                })
            }
            None => {
                // Not authenticated
                Ok(AuthorizationCheck {
                    authorized: false,
                    user: None,
                    required_role,
                    redirect_url: Some("/login".to_string()),
                })
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}

#[server]
pub async fn check_page_authorization(
    page_path: String,
) -> Result<AuthorizationCheck, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::server_functions::auth::get_current_user;

        let current_user = get_current_user().await?;

        match current_user {
            Some(user) => {
                let authorized = match page_path.as_str() {
                    "/dashboard" => true,
                    "/admindashboard" => user.is_admin(),
                    "/admintest" => user.is_teacher(),
                    "/assessments" => user.is_teacher(),
                    "/teachers" => user.is_teacher(),
                    "/gradebook" => user.is_teacher(),
                    "/testbuilder" => user.is_teacher(),
                    "/studentview" => user.is_admin(),
                    "/mathtesting" | "/readingtesting" => user.is_teacher(),
                    _ => true, // Default: allow access
                };

                let redirect_url = if !authorized {
                    if user.is_guest() {
                        Some("/login".to_string())
                    } else {
                        Some("/".to_string()) // Redirect to home if insufficient permissions
                    }
                } else {
                    None
                };

                Ok(AuthorizationCheck {
                    authorized,
                    user: Some(user),
                    required_role: None,
                    redirect_url,
                })
            }
            None => {
                // Not authenticated - redirect to login
                Ok(AuthorizationCheck {
                    authorized: false,
                    user: None,
                    required_role: None,
                    redirect_url: Some("/login".to_string()),
                })
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not implemented".to_string()))
    }
}
