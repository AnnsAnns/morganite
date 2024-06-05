use crate::protocol::Packet;


#[derive(Debug, Clone)]
pub enum ChannelEvent {
    Join(String),
    Leave(String),
    Message(String),
    Forward(Packet),
    Unknown
}