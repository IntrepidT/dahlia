#[cfg(feature = "ssr")]
use {
    crate::app::websockets::actors::ClientActor,
    actix::{Addr, Message},
};

// Message types
#[cfg(feature = "ssr")]
#[derive(Message)]
#[rtype(result = "()")]
pub struct ForwardMessage {
    pub client_id: String,
    pub message: String,
}
#[cfg(feature = "ssr")]
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub message: String,
}

// Register client with session
#[cfg(feature = "ssr")]
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterClient {
    pub client_id: String,
    pub client_addr: Addr<ClientActor>,
}
