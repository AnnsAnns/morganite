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

use crate::peer::Peer;
use crate::protocol::routing_packet::{RoutingEntry, RoutingPacket};
use crate::protocol::Packet;
use crate::protocol::{MESSAGE, CR, CRR, SCC, SCCR, STU};
use crate::protocol::shared_header::SharedHeader;
use crate::protocol::routed_packet::RoutedPacket;
use crate::shared::Shared;
use crate::{channel_events, swag_coding};

/// Process an individual chat client
pub async fn process(
    state: Arc<Mutex<Shared>>,
    stream: TcpStream,
    addr: SocketAddr,
    send_cr: bool,
) -> Result<(), Box<dyn Error>> {
    let local_addr = match stream.local_addr() {
        Ok(local_addr) => local_addr,
        Err(e) => { 
            tracing::info!("an error occurred; error = {:?}", e);
            panic!(); //TODO maybe different error handling
        },
    };
    let swag_coder = Framed::new(stream, SwagCoder::new());
    
    // Register our peer with state which internally sets up some channels.
    let mut peer = Peer::new(state.clone(), swag_coder).await?;
    // A client has connected, let's let everyone know.
    {
        let mut state = state.lock().await;
        let msg = tracing::info!("{addr} has joined the chat");
        state.broadcast(addr, &ChannelEvent::Join(addr.to_string())).await;
        if send_cr {
            match state.peers.get(&addr) {
                Some(entry) => {
                    if let Err(e) = entry.send(ChannelEvent::Routing(CR)) {
                        tracing::info!("Error sending the CR. error = {:?}", e);
                    }
                },
                None => { 
                    tracing::error!("Maybe too early for CR?: {}",addr);
                }
            };
        }
    }

    // Process incoming messages until our stream is exhausted by a disconnect.
    loop {
        tokio::select! {
            //another task send us a message:
            Some(event) = peer.rx.recv() => {
                tracing::info!("Received Event: {:#?}", event);
                // create packet
                let mut header = SharedHeader {
                    source_ip: local_addr.ip().to_string(),
                    source_port: local_addr.port().to_string(),
                    dest_ip: addr.ip().to_string(),
                    dest_port: addr.port().to_string(),
                    ttl: 16,
                };
                match event {
                    ChannelEvent::Message(msg, dest_addr) => {
                        header.dest_ip = dest_addr.ip().to_string();
                        header.dest_port = dest_addr.port().to_string();
                        let routed_packet = RoutedPacket {
                            header,
                            nickname: "TODO".to_string(),
                            message: msg,
                        };
                        peer.swag_coder.send(Packet::RoutedPacket(routed_packet)).await?;
                    },
                    ChannelEvent::Forward(packet) => {
                        peer.swag_coder.send(packet).await?;
                    }
                    ChannelEvent::Routing(type_id) => {
                        //get current routing table
                        let rt: Vec<RoutingEntry>;
                        {
                            let mut lock = state.lock().await;
                            rt = lock.get_routing_table(addr).await;
                        }
                        let routing_packet = RoutingPacket {
                            header,
                            table: rt,
                        };
                        tracing::info!("sending a routing packet.");
                        peer.swag_coder.send(Packet::RoutingPacket(routing_packet,type_id)).await?;
                    }
                    _ => tracing::error!("Received Event: {:#?} is not implemented!", event),
                } 
                


            }
            // A message was received from this tcp stream:
            result = peer.swag_coder.next() => match result {
                Some(Ok(packet)) => {
                    {
                        let mut state = state.lock().await;
                        tracing::info!("New Message from {}: {:#?}", addr, packet);
                        //handle message from others
                        state.broadcast(addr, &ChannelEvent::Unknown).await;
                    }

                    //assess what kind of packet we received:
                    match &packet {
                        Packet::RoutedPacket(routed_packet) => {
                            let mut new_event = ChannelEvent::Unknown;
                            //we received a message, check who's the destination:
                            let packet_destination = format!("{}:{}",routed_packet.header.dest_ip,routed_packet.header.dest_port);
                            tracing::info!("local: {}, dest: {}",local_addr, packet_destination);
                            match  packet_destination == local_addr.to_string() {
                                true => {
                                    //message is for us, display message
                                    tracing::info!("{}: {}",routed_packet.nickname, routed_packet.message);
                                },
                                //message is for someone else, try forwarding it:
                                false => {
                                    //parse destination to SocketAddr
                                    let destination_addr = match packet_destination.parse::<SocketAddr>() {
                                        Ok(socket) => socket,
                                        Err(e) => {
                                            tracing::error!("Error parsing destination to forward to: {}",e);
                                            continue;
                                        }
                                    };
                                    { //check routing table for the specified destination
                                        let mut lock = state.lock().await;
                                        //get next destination on route
                                        let routing_entry = match lock.routing_table.get(&destination_addr) {
                                            Some(routing_entry) => routing_entry,
                                            None => {
                                                tracing::error!("Forwarding: No route to destination available: {}",destination_addr);
                                                continue;
                                            }
                                        };
                                        //get channel to next on route
                                        let peer = match lock.peers.get(&routing_entry.next) {
                                            Some(peer) => peer,
                                            None => { 
                                                tracing::error!("Forwarding: No channel to destination available: {}",routing_entry.next);
                                                continue;
                                            }
                                        };
                                        //internal message to forward the packet as is
                                        new_event = ChannelEvent::Forward(packet);
                                        if let Err(e) = peer.send(new_event) {
                                            tracing::info!("Error sending your message. error = {:?}", e);
                                        }
                                    }
                                }
                            }
                        }
                        Packet::RoutingPacket(routing_packet, type_id) => {
                            tracing::info!("received a routing packet.");
                            //we received a routing packet, check which one and handle it:
                            match *type_id {
                                //routing packet type_ids:
                                CR => tracing::error!("Type ID of 2 not implemented" ),
                                CRR => tracing::error!("Type ID of 3 not implemented"),
                                SCC => tracing::error!("Type ID of 4 not implemented"),
                                SCCR => tracing::error!("Type ID 5 of not implemented"),
                                STU => tracing::error!("Type ID 6 of not implemented"),
                                //undefined type_id:
                                MESSAGE => tracing::error!("Routing packet with type_id of Message detected!"),
                                _ => tracing::error!("Type ID not implemented or expected!"),
                            }
                        }
                    }
                }
                // An error occurred.
                Some(Err(e)) => {
                    tracing::error!(
                        "an error occurred while processing messages for {}; error = {:?}",
                        addr,
                        e
                    );
                }
                // The stream has been exhausted.
                None => break,
            },
        }
    }

    // If this section is reached it means that the client was disconnected!
    // Let's let everyone still connected know about it.
    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);
        state.routing_table.remove(&addr);

        let msg = format!("{} has left the chat", addr);
        tracing::info!("{}", msg);
        state.broadcast(addr, &ChannelEvent::Leave(addr.to_string())).await;
    }

    Ok(())
}