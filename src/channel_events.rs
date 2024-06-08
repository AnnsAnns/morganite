use std::net::SocketAddr;

use crate::protocol::Packet;


#[derive(Debug, Clone)]
pub enum ChannelEvent {
    Join(String), //current thoughts: Terminal output for Join and Leave only in console(if not when initially receiving the message) 
    Leave(String), 
    Message(String, SocketAddr), //only for sending messages to other clients?
    Forward(Packet), 
    Unknown
}