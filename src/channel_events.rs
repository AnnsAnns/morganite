use std::collections::HashMap;
use std::net::SocketAddr;

use std::sync::mpsc::Sender;

use crate::protocol::Packet;
use crate::shared::RoutingTableEntry;

#[derive(Debug, Clone, PartialEq)]
pub enum Commands {
    Connect(SocketAddr),
    Contacts,
    Message(SocketAddr, String),
    Quit,
    Help,
    Unknown(String),
    SetOwnNick(String),
    Broadcast(String)
}

#[derive(Debug, Clone)]
pub enum ChannelEvent {
    Join(String), //current thoughts: Terminal output for Join and Leave only in console(if not when initially receiving the message)
    Leave(String),
    Message(String, SocketAddr), //message, destination
    Routing(u8),                 //type id
    Forward(Packet),
    Command(Commands),
    Contacts(HashMap<SocketAddr, RoutingTableEntry>),
    CommandReceiver(Sender<ChannelEvent>),
    MessageToTUI(String, String, SocketAddr), //message, sender, destination
    LogToTerminal(String),
}
