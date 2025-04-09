use crate::app::models::session::{SessionEvent, SessionMessage, UpdateTestSessionRequest};
use chrono::Utc;
use serde_json::from_str;
use std::collections::HashMap;

#[cfg(feature = "ssr")]
use {
    crate::app::db::session_database,
    crate::app::db::session_database::update_session,
    crate::app::server_functions::sessions,
    crate::app::websockets::messages::{ClientMessage, ForwardMessage, RegisterClient},
    actix::{Actor, ActorContext, Addr, AsyncContext, Handler, StreamHandler},
    actix_web_actors::ws,
};

// Session actor with database integration
#[cfg(feature = "ssr")]
pub struct TestSessionActor {
    pub id: String,
    pub clients: HashMap<String, Addr<ClientActor>>,
}

#[cfg(feature = "ssr")]
impl Actor for TestSessionActor {
    type Context = actix::Context<Self>;
}

// Client actor
#[cfg(feature = "ssr")]
pub struct ClientActor {
    pub id: String,
    pub session_id: String,
    pub role: String, // "teacher" or "student"
    pub session_addr: Addr<TestSessionActor>,
}

#[cfg(feature = "ssr")]
impl Actor for ClientActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.session_addr.do_send(RegisterClient {
            client_id: self.id.clone(),
            client_addr: addr,
        });
    }
}

// Implementation of WebSocket protocol for client actor
#[cfg(feature = "ssr")]
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ClientActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Parse incoming messages
                if let Ok(session_msg) = from_str::<SessionMessage>(&text) {
                    // Update database state based on message type
                    let session_id = self.session_id.clone();
                    let role = self.role.clone();

                    // Only process messages from teachers
                    if role == "teacher" {
                        // Handle database updates in a separate task
                        actix::spawn(async move {
                            match session_msg.event {
                                SessionEvent::NavigateToCard(index) => {
                                    // Update current card index in database
                                    let _ = sessions::update_session_state(
                                        session_id,
                                        Some(index as i32),
                                        None,
                                        None,
                                    )
                                    .await;
                                }
                                SessionEvent::CompleteSession => {
                                    // Mark session as completed
                                    let _ = sessions::update_session_state(
                                        session_id,
                                        None,
                                        Some(false),
                                        Some(Utc::now()),
                                    )
                                    .await;
                                }
                                _ => {} // Other event types don't affect database state
                            }
                        });
                    }

                    // Forward message to session actor for broadcast
                    self.session_addr.do_send(ForwardMessage {
                        client_id: self.id.clone(),
                        message: text.to_string(),
                    });
                }
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

// Handler for ForwardMessage in TestSessionActor
#[cfg(feature = "ssr")]
impl Handler<ForwardMessage> for TestSessionActor {
    type Result = ();

    fn handle(&mut self, msg: ForwardMessage, _: &mut Self::Context) {
        // Broadcast message to all connected clients except sender
        for (client_id, client) in &self.clients {
            if client_id != &msg.client_id {
                client.do_send(ClientMessage {
                    message: msg.message.clone(),
                });
            }
        }
    }
}

// Handler for ClientMessage in ClientActor
#[cfg(feature = "ssr")]
impl Handler<ClientMessage> for ClientActor {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) {
        ctx.text(msg.message);
    }
}

// Handler for RegisterClient in TestSessionActor
#[cfg(feature = "ssr")]
impl Handler<RegisterClient> for TestSessionActor {
    type Result = ();

    fn handle(&mut self, msg: RegisterClient, _: &mut Self::Context) {
        self.clients.insert(msg.client_id, msg.client_addr);
    }
}
