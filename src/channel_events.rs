use crate::protocol::Packet;


#[derive(Debug)]
pub enum ChannelEvent {
    Join,
    Leave,
    Message(String),
    Forward(Packet),
}