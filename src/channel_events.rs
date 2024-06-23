use std::net::SocketAddr;

use crate::protocol::Packet;

#[derive(Debug, Clone)]
pub enum Commands {
    Connect(SocketAddr),
    Contacts,
    Message(SocketAddr, String),
    Quit,
    Unknown(String),
}


#[derive(Debug, Clone)]
pub enum ChannelEvent {
    Join(String), //current thoughts: Terminal output for Join and Leave only in console(if not when initially receiving the message) 
    Leave(String), 
    Message(String, SocketAddr), //message, destination
    Routing(u8), //type id
    Forward(Packet), 
    Command(Commands),
    Unknown
}