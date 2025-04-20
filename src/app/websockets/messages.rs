#[cfg(feature = "ssr")]
use actix::prelude::{Message, Recipient};
use uuid::Uuid;

#[cfg(feature = "ssr")]
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[cfg(feature = "ssr")]
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub lobby_id: Uuid,
    pub self_id: Uuid,
}

#[cfg(feature = "ssr")]
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
    pub room_id: Uuid,
}

#[cfg(feature = "ssr")]
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub id: Uuid,
    pub msg: String,
    pub room_id: Uuid,
}
