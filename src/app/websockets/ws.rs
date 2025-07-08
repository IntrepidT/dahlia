#[cfg(feature = "ssr")]
use {
    crate::app::websockets::lobby::Lobby,
    crate::app::websockets::messages::{
        ClientActorMessage, Connect, Disconnect, TestMessageType, TestSessionMessage,
        UserInfoMessage, WsMessage,
    },
    actix::{fut, ActorContext, ActorFuture, ContextFutureSpawner, WrapFuture},
    actix::{Actor, Addr, Running, StreamHandler},
    actix::{ActorFutureExt, AsyncContext, Handler},
    actix_web_actors::ws,
    serde_json::{from_str, json, Value}, // Added json! macro here
};

use std::time::{Duration, Instant};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[cfg(feature = "ssr")]
pub struct WsConn {
    room: Uuid,
    lobby_addr: Addr<Lobby>,
    hb: Instant,
    id: Uuid,
}

#[cfg(feature = "ssr")]
impl WsConn {
    pub fn new(room: Uuid, lobby: Addr<Lobby>) -> WsConn {
        WsConn {
            id: Uuid::new_v4(),
            room,
            hb: Instant::now(),
            lobby_addr: lobby,
        }
    }
}

#[cfg(feature = "ssr")]
impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.lobby_addr
            .send(Connect {
                addr: addr.recipient(),
                lobby_id: self.room,
                self_id: self.id,
                user_role: None,           //Role assigned later
                is_session_creator: false, //this is determined based on logic
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx)
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.lobby_addr.do_send(Disconnect {
            id: self.id,
            room_id: self.room,
        });
        Running::Stop
    }
}

#[cfg(feature = "ssr")]
impl WsConn {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting failed heartbeat");
                ctx.stop();
                return;
            }

            ctx.ping(b"hi");
        });
    }
}

#[cfg(feature = "ssr")]
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(ws::Message::Text(s)) => {
                log::info!("Received WebSocket message: {}", s);

                if let Ok(json_value) = from_str::<Value>(&s) {
                    if let Some(msg_type) = json_value.get("type").and_then(|t| t.as_str()) {
                        log::info!("Processing message type: {}", msg_type);

                        match msg_type {
                            "user_info" => {
                                log::info!("Handling user_info from user {}", self.id);
                                // Send user info to lobby for role assignment
                                self.lobby_addr.do_send(UserInfoMessage {
                                    user_data: json_value,
                                    user_id: self.id,
                                    room_id: self.room,
                                });
                                return;
                            }
                            "user_info" => {
                                log::info!("Handling user info from user {}", self.id);
                                //Send user info to lobby for role assignment
                                self.lobby_addr.do_send(UserInfoMessage {
                                    user_data: json_value,
                                    user_id: self.id,
                                    room_id: self.room,
                                });
                                return;
                            }
                            "request_participants" => {
                                log::info!("Handling request_participants from user {}", self.id);
                                self.lobby_addr.do_send(TestSessionMessage {
                                    message_type: TestMessageType::RequestParticipants,
                                    payload: json!({}),
                                    id: self.id,
                                    room_id: self.room,
                                });
                                return;
                            }
                            "test_message" => {
                                if let Some(test_msg_type_str) =
                                    json_value.get("test_message_type").and_then(|t| t.as_str())
                                {
                                    log::info!(
                                        "Processing test message type: {}",
                                        test_msg_type_str
                                    );

                                    let message_type = match test_msg_type_str {
                                        "start_test" => TestMessageType::StartTest,
                                        "submit_answer" => TestMessageType::SubmitAnswer,
                                        "teacher_comment" => TestMessageType::TeacherComment,
                                        "end_test" => TestMessageType::EndTest,
                                        "student_joined" => TestMessageType::StudentJoined,
                                        "student_left" => TestMessageType::StudentLeft,
                                        "question_focus" => TestMessageType::QuestionFocus,
                                        "time_update" => TestMessageType::TimeUpdate,
                                        unknown => {
                                            log::warn!("Unknown test message type: {}", unknown);
                                            return;
                                        }
                                    };

                                    let payload =
                                        json_value.get("payload").cloned().unwrap_or(Value::Null);

                                    self.lobby_addr.do_send(TestSessionMessage {
                                        message_type,
                                        payload,
                                        id: self.id,
                                        room_id: self.room,
                                    });
                                    return;
                                }
                            }
                            _ => {
                                log::info!("Unhandled message type: {}", msg_type);
                            }
                        }
                    } else {
                        log::warn!("Message missing 'type' field: {}", s);
                    }
                } else {
                    log::warn!("Failed to parse JSON message: {}", s);
                }

                // Fallback to regular chat message
                self.lobby_addr.do_send(ClientActorMessage {
                    id: self.id,
                    msg: s.to_string(),
                    room_id: self.room,
                });
            }
            Err(e) => {
                log::error!("WebSocket protocol error: {:?}", e);
                std::panic::panic_any(e)
            }
        }
    }
}

#[cfg(feature = "ssr")]
impl Handler<WsMessage> for WsConn {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
