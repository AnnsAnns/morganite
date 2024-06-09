use channel_events::ChannelEvent;
use swag_coding::SwagCoder;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, FramedRead, LinesCodec};

use futures::SinkExt;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::protocol::routing_packet::RoutingEntry;
use crate::{channel_events, swag_coding};

/// Shorthand for the transmit half of the message channel.
pub type Tx = mpsc::UnboundedSender<ChannelEvent>;

/// Shorthand for the receive half of the message channel.
pub type Rx = mpsc::UnboundedReceiver<ChannelEvent>;

pub struct RoutingTableEntry {
    pub next: SocketAddr,
    pub hop_count: i32,
    pub ttl: bool, //or timestamp? TODO
}
/// Data that is shared between all peers in the chat server.
///
/// This is the set of `Tx` handles for all connected clients. Whenever a
/// message is received from a client, it is broadcasted to all peers by
/// iterating over the `peers` entries and sending a copy of the message on each
/// `Tx`.
pub struct Shared {  
    pub peers: HashMap<SocketAddr, Tx>, //maybe refactor to maybe channels or streams?
    //                         target    |  next,hop_count,ttl
    pub routing_table: HashMap<SocketAddr, RoutingTableEntry>,
}


impl Shared {
    /// Create a new, empty, instance of `Shared`.
    pub fn new() -> Self {
        Shared {
            peers: HashMap::new(),
            routing_table: HashMap::new(),
        }
    }

    /// Send a `LineCodec` encoded message to every peer, except
    /// for the sender.
    pub async fn broadcast(&mut self, sender: SocketAddr, event: &ChannelEvent) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(event.clone());
            }
        }
    }
    /// Return all entries in the routing table besides the ones with target as destination or next as a vector 
    pub async fn get_routing_table(&mut self,target: SocketAddr) -> Vec<RoutingEntry> {
        let mut routing_entries: Vec<RoutingEntry> = Vec::new();                                                        //entry is the direct connection or entry is reachable through the target
        for entry in self.routing_table.iter().filter(|(dest,rt_entry)| **dest != target && rt_entry.next != target) {
            routing_entries.push(RoutingEntry {
                target_ip: entry.0.ip().to_string(),
                target_port: entry.0.port(),
                next_ip: entry.1.next.ip().to_string(),
                next_port: entry.1.next.port(),
                hop_count: entry.1.hop_count,
            });
        }
        routing_entries
    }
}
#[tokio::test]
pub async fn test_get_routing_table(){
    let target = "127.0.0.1:6666".parse::<SocketAddr>().unwrap();
    let mut shared = Shared::new();
    shared.routing_table.insert("127.0.0.1:12345".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:12346".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    shared.routing_table.insert("127.0.0.1:6666".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:1236".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    shared.routing_table.insert("127.0.0.1:1235".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:6666".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    shared.routing_table.insert("127.0.0.1:12344".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:6666".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    
    assert_eq!(vec![RoutingEntry{target_ip: "127.0.0.1".to_string(),target_port:12345,next_ip:"127.0.0.1".to_string(),next_port:12346,hop_count:2}],shared.get_routing_table(target).await);
}