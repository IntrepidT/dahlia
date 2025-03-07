use crate::app::models::user::User;

// Server-specific implementation gated behind "ssr" feature
#[cfg(feature = "ssr")]
mod server {
    use actix_web::{
        dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
        Error, HttpMessage,
    };
    use futures::future::{ready, LocalBoxFuture, Ready};
    use std::rc::Rc;
    use std::sync::Arc;
    use std::task::{Context, Poll};

    use crate::app::models::user::User;

    pub struct Authentication {
        password_hash: Arc<String>,
    }

    impl Authentication {
        pub fn new(password_hash: String) -> Self {
            Authentication {
                password_hash: Arc::new(password_hash),
            }
        }
    }

    impl<S, B> Transform<S, ServiceRequest> for Authentication
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<B>;
        type Error = Error;
        type Transform = AuthenticationMiddleware<S>;
        type InitError = ();
        type Future = Ready<Result<Self::Transform, Self::InitError>>;

        fn new_transform(&self, service: S) -> Self::Future {
            ready(Ok(AuthenticationMiddleware {
                service: Rc::new(service),
                password_hash: self.password_hash.clone(),
            }))
        }
    }

    pub struct AuthenticationMiddleware<S> {
        service: Rc<S>,
        password_hash: Arc<String>,
    }

    impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<B>;
        type Error = Error;
        type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

        forward_ready!(service);

        fn call(&self, req: ServiceRequest) -> Self::Future {
            let service = self.service.clone();
            let password_hash = self.password_hash.clone();

            Box::pin(async move {
                // Get session from request extensions
                if let Some(session) = req.extensions().get::<Arc<String>>() {
                    // Verify session token
                    // For now, just check if the session exists
                    let user_id = validate_session(session.to_string()).await;
                    if let Some(user_id) = user_id {
                        // Load user from database
                        let user = get_user_by_id(user_id).await;
                        if let Some(user) = user {
                            // Add user to request extensions
                            req.extensions_mut().insert(user);
                        }
                    }
                }

                // Continue with the request
                let res = service.call(req).await?;
                Ok(res)
            })
        }
    }

    // Placeholder function to validate a session
    async fn validate_session(session_token: String) -> Option<i64> {
        // In a real application, look up session in database
        // For now, just return a placeholder user ID if the session is not empty
        if !session_token.is_empty() {
            Some(1)
        } else {
            None
        }
    }

    // Placeholder function to get user by ID
    async fn get_user_by_id(user_id: i64) -> Option<User> {
        // In a real application, look up user in database
        // For now, just return a placeholder user
        Some(User {
            id: user_id,
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            role: "user".to_string(),
        })
    }
}

//Reexport server types when "ssr" feature is enabled
#[cfg(feature = "ssr")]
pub use server::*;
