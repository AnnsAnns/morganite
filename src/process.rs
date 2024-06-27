use channel_events::ChannelEvent;

use swag_coding::SwagCoder;
use tokio::net::{TcpStream};
use tokio::sync::{Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed};

use futures::SinkExt;


use std::error::Error;

use std::net::SocketAddr;
use std::sync::Arc;


use crate::heartbeat::POISE_UNREACHABLE;
use crate::peer::Peer;
use crate::protocol::routed_packet::RoutedPacket;
use crate::protocol::routing_packet::{RoutingPacket};
use crate::protocol::shared_header::SharedHeader;
use crate::protocol::Packet;
use crate::protocol::{CR, CRR, MESSAGE, SCC, SCCR, STU};
use crate::shared::{RoutingTableEntry, Shared};
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
        }
    };
    let listener_address = state.lock().await.listener_addr.clone().parse::<SocketAddr>().unwrap();
    let swag_coder = Framed::new(stream, SwagCoder::new());

    // Register our peer with state which internally sets up some channels.
    let mut peer = Peer::new(state.clone(), swag_coder).await?;
    // A client has connected, let's let everyone know.
    {
        let mut state = state.lock().await;
        let msg = tracing::info!("{addr} has joined the chat");
        state
            .broadcast(addr, &ChannelEvent::Join(addr.to_string()))
            .await;
        if send_cr {
            match state.peers.get(&addr) {
                Some(entry) => {
                    if let Err(e) = entry.send(ChannelEvent::Routing(CR)) {
                        tracing::info!("Error sending the CR. error = {:?}", e);
                    }
                }
                None => {
                    tracing::error!("Maybe too early for CR?: {}", addr);
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
                    //source addr is listener!
                    source_ip: listener_address.ip().to_string(),
                    source_port: listener_address.port(),
                    dest_ip: addr.ip().to_string(),
                    dest_port: addr.port(),
                    ttl: 16,
                };
                match event {
                    ChannelEvent::Message(msg, dest_addr) => {
                        header.dest_ip = dest_addr.ip().to_string();
                        header.dest_port = dest_addr.port();
                        // Get the nickname
                        let nickname = {
                            let state = state.lock().await;
                            state.nickname.clone()
                        };
                        let routed_packet = RoutedPacket {
                            header,
                            nickname,
                            message: msg,
                        };
                        peer.swag_coder.send(Packet::RoutedPacket(routed_packet)).await?;
                    },
                    //we received a message to be forwarded
                    ChannelEvent::Forward(mut packet) => {
                        //decrease ttl and send package
                        match &mut packet {
                            Packet::RoutedPacket(routed_packet) => {
                                routed_packet.header.ttl -= 1;
                            },
                            _ => {},
                        }
                        peer.swag_coder.send(packet).await?;
                    }
                    ChannelEvent::Routing(type_id) => {
                        //get current routing table
                        let rt = if type_id != SCC {
                            let mut lock = state.lock().await;
                            lock.get_routing_table(addr,local_addr).await
                        } else {
                            Vec::new()
                        };

                        let routing_packet = RoutingPacket {
                            header,
                            table: Some(rt),
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
                        let state = state.lock().await;
                        tracing::info!("New Packet from {}: {:#?}", addr, packet);
                        state.console_input_sender.send(ChannelEvent::LogToTerminal(format!("New Packet from {}: {:?}", addr, packet))).unwrap();
                    }

                    //assess what kind of packet we received:
                    match &packet {
                        Packet::RoutedPacket(routed_packet) => {
                            //we received a message, check who's the destination:
                            let packet_destination = format!("{}:{}",routed_packet.header.dest_ip,routed_packet.header.dest_port);
                            match  packet_destination == local_addr.to_string() {
                                true => {
                                    //message is for us, display message
                                    tracing::info!("{}: {}",routed_packet.nickname, routed_packet.message);
                                    // Send a broadcast to inform everyone about the message
                                    {
                                        let mut state = state.lock().await;
                                        state.broadcast(addr, &ChannelEvent::MessageToTUI(routed_packet.message.clone(), routed_packet.nickname.clone(), addr)).await;
                                    }
                                },
                                //message is for someone else, try forwarding it:
                                false => {
                                    if routed_packet.header.ttl > 0 {
                                        //parse destination to SocketAddr
                                        let destination_addr = match packet_destination.parse::<SocketAddr>() {
                                            Ok(socket) => socket,
                                            Err(e) => {
                                                tracing::error!("Error parsing destination to forward to: {}",e);
                                                continue;
                                            }
                                        };
                                        { //check routing table for the specified destination
                                            let lock = state.lock().await;
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
                                            let new_event = ChannelEvent::Forward(packet);
                                            if let Err(e) = peer.send(new_event) {
                                                tracing::info!("Error sending your message. error = {:?}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Packet::RoutingPacket(routing_packet, type_id) => {
                            tracing::info!("received a routing packet.");
                            //we received a routing packet, check which one and handle it:
                            let reply_header = SharedHeader {
                                source_ip: listener_address.ip().to_string(),
                                source_port: listener_address.port(),
                                dest_ip: addr.ip().to_string(),
                                dest_port: addr.port(),
                                ttl: 16,
                            };

                            let routingtable = match &routing_packet.table {
                                Some(table) => table.clone(),
                                None => Vec::new(),
                            };

                            match *type_id {
                                //routing packet type_ids:
                                CR | STU => { 
                                    if *type_id == CR {
                                        //Add connection to routing table with source ip + port as target and stream address as next
                                        let target_address: SocketAddr = (routing_packet.header.source_ip.clone() + ":" + &routing_packet.header.source_port.to_string()).parse::<SocketAddr>().unwrap();
                                        let mut lock = state.lock().await;
                                        lock.routing_table.insert(target_address, RoutingTableEntry {next:addr, hop_count: 1, ttl: true});
                                    }
                                    //need to send a reply containing the routing table:
                                    let reply_table;
                                    {
                                        let mut lock = state.lock().await;
                                        lock.update_routing_table(routingtable,addr).await;
                                        reply_table = lock.get_routing_table(addr, local_addr).await;
                                    }
                                    let reply_routing_packet: RoutingPacket = RoutingPacket{header: reply_header.clone(), table: Some(reply_table)};
                                    if *type_id == CR || *type_id == SCC{
                                        let id = type_id + 1;
                                        //send CRR or SCCR
                                        tracing::info!("replying to CR or SCC: {type_id} with {id} to {:?}.", reply_header);
                                        peer.swag_coder.send(Packet::RoutingPacket(reply_routing_packet, id)).await?;
                                    }
                                },
                                CRR => {
                                    //update routing table based on received information:
                                    let mut lock = state.lock().await;
                                    lock.update_routing_table(routingtable, addr).await;
                                },
                                SCC => {
                                    // Send a SCCR to the sender
                                    tracing::info!("replying to SCC with SCCR to {:?}.", reply_header);
                                    let reply_routing_packet: RoutingPacket = RoutingPacket{header: reply_header.clone(), table: Some(Vec::new())};
                                    peer.swag_coder.send(Packet::RoutingPacket(reply_routing_packet, SCCR)).await?;
                                }
                                SCCR => {
                                    // Mark the sender as responding:
                                    let mut lock = state.lock().await;
                                    let target_address: SocketAddr = (routing_packet.header.source_ip.clone() + ":" + &routing_packet.header.source_port.to_string()).parse::<SocketAddr>().unwrap();
                                    lock.routing_table.entry(target_address).and_modify(|rt_entry| rt_entry.ttl = true);
                                },
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

        // Poise reverse routing table
        for (dest, rt_entry) in state.routing_table.iter_mut() {
            if rt_entry.next == addr {
                rt_entry.hop_count = POISE_UNREACHABLE;
            }
        }

        let msg = format!("{} has left the chat", addr);
        tracing::info!("{}", msg);
        state
            .broadcast(addr, &ChannelEvent::Leave(addr.to_string()))
            .await;
    }

    Ok(())
}
