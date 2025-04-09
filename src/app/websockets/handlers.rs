use crate::app::server_functions::sessions::get_session;
use std::collections::HashMap;
use uuid::Uuid;
#[cfg(feature = "ssr")]
use {
    crate::app::websockets::actors::{ClientActor, TestSessionActor},
    crate::app::websockets::messages::RegisterClient,
    crate::app::websockets::state::AppState,
    actix::Actor,
    actix_web::{web, Error, HttpRequest, HttpResponse},
    actix_web_actors::ws,
};

// WebSocket connection handler with role specification
#[cfg(feature = "ssr")]
pub async fn ws_session(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<(String, String)>, // (session_id, role)
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (session_id, role) = path.into_inner();
    let session_id_clone = session_id.clone();

    // Verify session exists in database
    match get_session(session_id).await {
        Ok(Some(session)) => {
            // Don't allow connections to inactive sessions
            if !session.is_active {
                return Ok(HttpResponse::BadRequest().body("Session is no longer active"));
            }

            // Get or create session actor
            let session_addr = {
                let mut sessions = data.sessions.lock().unwrap();
                let session_id_for_actor = session_id_clone.clone();
                sessions
                    .entry(session_id_clone.clone())
                    .or_insert_with(|| {
                        let actor = TestSessionActor {
                            id: session_id_for_actor,
                            clients: HashMap::new(),
                        };
                        actor.start()
                    })
                    .clone()
            };

            // Create new client actor
            let client = ClientActor {
                id: Uuid::new_v4().to_string(),
                session_id: session_id_clone,
                role: role.clone(),
                session_addr: session_addr.clone(),
            };

            // Start WebSocket connection
            ws::start(client, &req, stream)
        }
        Ok(None) => Ok(HttpResponse::NotFound().body("Session not found")),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Database error")),
    }
}

// Setup function
#[cfg(feature = "ssr")]
pub fn configure_websocket(cfg: &mut web::ServiceConfig) {
    let app_state = web::Data::new(AppState::new());

    cfg.app_data(app_state.clone()).route(
        "/api/test-session/{session_id}/{role}",
        web::get().to(ws_session),
    );
}
