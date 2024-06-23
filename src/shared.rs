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
use crate::{channel_events, console_middleware, swag_coding};

/// Shorthand for the transmit half of the message channel.
pub type Tx = mpsc::UnboundedSender<ChannelEvent>;

/// Shorthand for the receive half of the message channel.
pub type Rx = mpsc::UnboundedReceiver<ChannelEvent>;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub console_input_sender: Tx,
    pub console_cmd_receiver: Arc<Mutex<Rx>>,
    //                         target    |  next,hop_count,ttl
    pub routing_table: HashMap<SocketAddr, RoutingTableEntry>,
}


impl Shared {
    /// Create a new, empty, instance of `Shared`.
    pub fn new(console_input_sender: Tx, console_cmd_receiver: Rx) -> Self {
        // See console_middleware.rs for more information on this
        let console_cmd_safe = Arc::new(Mutex::new(console_cmd_receiver));
        
        Shared {
            peers: HashMap::new(),
            routing_table: HashMap::new(),
            console_input_sender,
            console_cmd_receiver: console_cmd_safe,
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
    pub async fn get_routing_table(&mut self,target: SocketAddr,local: SocketAddr) -> Vec<RoutingEntry> {
        let mut routing_entries: Vec<RoutingEntry> = Vec::new();                                                        //entry is the direct connection or entry is reachable through the target
        for entry in self.routing_table.iter().filter(|(dest,rt_entry)| **dest != target && rt_entry.next != target) {
            routing_entries.push(RoutingEntry {
                target_ip: entry.0.ip().to_string(),
                target_port: entry.0.port(),
                next_ip: local.ip().to_string(), //our address since we only add connections through us to the update
                next_port: local.port(), //replace with const we set in main?
                hop_count: entry.1.hop_count+1,
            });
        }
        routing_entries
    }
    /// updates the routing table with the given information
    pub async fn update_routing_table(&mut self, update: Vec<RoutingEntry>) {
        for new_entry in update.iter() {
            // get target
            let target = (new_entry.target_ip.clone()+":"+&new_entry.target_port.to_string()).parse::<SocketAddr>().unwrap();

            match self.routing_table.get(&target) {
                // if in Routing Table
                Some(old_entry) => {
                    // compare hop_count to target in Routing Table and in update
                    let hop_count = new_entry.hop_count;
                    if hop_count <= old_entry.hop_count {
                        let next =(new_entry.next_ip.clone()+":"+&new_entry.next_port.to_string()).parse::<SocketAddr>().unwrap();
                        // if update is shorter: replace/change entry in Routing Table
                        self.routing_table.insert(target, RoutingTableEntry{next,hop_count,ttl: true});
                    }
                }
                // if not in Routing Table: create new entry to target through source
                None => {
                    let hop_count = new_entry.hop_count;
                    let next =(new_entry.next_ip.clone()+":"+&new_entry.next_port.to_string()).parse::<SocketAddr>().unwrap();
                    self.routing_table.insert(target, RoutingTableEntry{next,hop_count,ttl: true});
                }
            }
        }
    }
}
#[tokio::test]
pub async fn test_get_routing_table(){
    let target = "127.0.0.1:6666".parse::<SocketAddr>().unwrap();
    let local = "127.0.0.1:6142".parse::<SocketAddr>().unwrap();
    let (fake_tx, fake_rx) = mpsc::unbounded_channel();
    let mut shared = Shared::new(fake_tx, fake_rx);
    shared.routing_table.insert("127.0.0.1:12345".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:12346".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    shared.routing_table.insert("127.0.0.1:6666".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:1236".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    shared.routing_table.insert("127.0.0.1:1235".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:6666".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    shared.routing_table.insert("127.0.0.1:12344".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:6666".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    
    assert_eq!(vec![RoutingEntry{target_ip: "127.0.0.1".to_string(),target_port:12345,next_ip:"127.0.0.1".to_string(),next_port:6142,hop_count:3}],shared.get_routing_table(target,local).await);
    
    let update = vec![RoutingEntry{target_ip: "127.0.0.1".to_string(),target_port:11111,next_ip:"127.0.0.1".to_string(),next_port:12345,hop_count:3},
        RoutingEntry{target_ip: "127.0.0.1".to_string(),target_port:11112,next_ip:"127.0.0.1".to_string(),next_port:12345,hop_count:4},
        RoutingEntry{target_ip: "127.0.0.1".to_string(),target_port:11113,next_ip:"127.0.0.1".to_string(),next_port:12345,hop_count:5}];
    shared.update_routing_table(update).await;
    //vergleichsmap
    let mut rt: HashMap<SocketAddr, RoutingTableEntry> = HashMap::new();
    rt.insert("127.0.0.1:12345".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:12346".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    rt.insert("127.0.0.1:6666".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:1236".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    rt.insert("127.0.0.1:1235".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:6666".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    rt.insert("127.0.0.1:12344".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:6666".parse::<SocketAddr>().unwrap(), hop_count:2, ttl:true});
    rt.insert("127.0.0.1:11111".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:12345".parse::<SocketAddr>().unwrap(), hop_count:3, ttl:true});
    rt.insert("127.0.0.1:11112".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:12345".parse::<SocketAddr>().unwrap(), hop_count:4, ttl:true});
    rt.insert("127.0.0.1:11113".parse::<SocketAddr>().unwrap(),RoutingTableEntry{next:"127.0.0.1:12345".parse::<SocketAddr>().unwrap(), hop_count:5, ttl:true});

    assert_eq!(shared.routing_table,rt);
}