use crate::packets::connection::ConnectionPacket;
use crate::packets::header::{BaseHeader, PacketType, BASE_HEADER_SIZE};
use crate::packets::message::MessagePacket;
use crate::packets::routing_entry::RoutingEntry;
use crate::packets::Packet;
use crate::Morganite;

use bytes::BytesMut;
use log::{debug, error, info};

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use super::socket::SocketStream;

pub struct SocketReadHandler {
    pub morganite: Arc<Mutex<Morganite>>,
    socket: Arc<Mutex<SocketStream>>,
    target_name: String,
    peer_addr: String,
    own_addr: String,
}

impl SocketReadHandler {
    pub fn new(morganite: Arc<Mutex<Morganite>>, socket: Arc<Mutex<SocketStream>>, peer_addr: String, own_addr: String) -> SocketReadHandler {
        SocketReadHandler {
            morganite,
            socket,
            target_name: "".to_string(),
            peer_addr,
            own_addr,
        }
    }

    pub async fn process(&mut self) {
        loop {
            let mut msg = Vec::new();

            match self.socket.lock().await.read_to_end(&mut msg).await {
                Ok(n) => {
                    if n == 0 {
                        info!("Connection to {} closed by client!", self.target_name);
                        return;
                    }
                    debug!("Received {} bytes", n);
                    debug!("Received: {:?}", msg);
                    // get first byte to determine type of message
                    let msg_type = match PacketType::from_u8(msg[0]) {
                        Some(msg_type) => msg_type,
                        None => {
                            error!("Unknown message type: {}", msg[0]);
                            return;
                        }
                    };

                    let packet = Packet::from_bytes(msg);

                    if !packet.verify_self().await {
                        error!("Packet verification failed!");
                        return;
                    } else {
                        debug!("Packet verification successful!");
                    }

                    let header = BaseHeader::from_bytes(packet.bytes.clone()).unwrap();

                    // Check if packet is for this node otherwise we forward it
                    if header.get_target() != self.morganite.lock().await.get_own_name() {
                        debug!("Packet target is not this node! Forwarding ...");

                        // Forward packet
                        let morganite = self.morganite.lock().await;
                        let addr = match morganite.get_addr_of(header.get_target()).await {
                            Some(addr) => addr,
                            None => {
                                error!("Could not find IP of target to forward packet to!");
                                return;
                            }
                        };

                        let mut stream = TcpStream::connect(addr.clone()).await.unwrap();
                        stream.write_all(&packet.to_bytes()).await.unwrap();
                        stream.flush().await.unwrap();
                        debug!("Packet forwarded to {}", addr);
                        stream.shutdown().await.unwrap();
                        continue;
                    }

                    // Reset ttl to 30
                    self.morganite
                        .lock()
                        .await
                        .routingtable
                        .lock()
                        .await
                        .reset_ttl_for_target(header.get_source());

                    match msg_type {
                        PacketType::Connection => {
                            // Connection message
                            debug!("Connection message received");

                            let bytes = packet.bytes;

                            let _base_header = BaseHeader::from_bytes(bytes.clone()).unwrap();
                            let connection_packet = ConnectionPacket::from_bytes(BytesMut::from(
                                // Create BytesMut
                                bytes[BASE_HEADER_SIZE..] // Get remaining bytes (without header)
                                    .to_vec()
                                    .as_slice(), // Convert Vec<u8> to &[u8]
                            ));
                    
                            let mut morganite = self.morganite.lock().await;
                            // As this client directly connected to us we can ignore other routing entries to that client
                            morganite.remove_entry(connection_packet.name.clone()).await;
                    
                            let peer_addr = self.peer_addr.clone().to_string();
                            let full_addr = peer_addr.split(':').collect::<Vec<&str>>();
                            let ip = full_addr.first().unwrap().to_string();
                            let _port = full_addr.get(1).unwrap().parse::<u16>().unwrap();
                            self.target_name = connection_packet.name.clone();
                    
                            debug!("Adding routing entry for {}", connection_packet.name);
                    
                            let own_name = morganite.get_own_name();
                    
                            morganite
                                .routingtable_add(RoutingEntry::new(
                                    own_name,
                                    connection_packet.name.clone(),
                                    ip,
                                    connection_packet.port,
                                    1, // Is it 2?
                                ))
                                .await;
                            debug!("Added routing entry for {}", connection_packet.name);
                        }
                        PacketType::Routing => {
                            // Routing message
                            debug!("Routing message received");
                            self.morganite
                                .lock()
                                .await
                                .update_routing_table(
                                    packet.bytes,
                                    self.peer_addr
                                        .to_string()
                                        .split(':')
                                        .collect::<Vec<&str>>()
                                        .first()
                                        .unwrap()
                                        .to_string(), // Get the IP, I know this is ugly
                                )
                                .await;
                        }
                        PacketType::Message => {
                            // Data message
                            debug!("Data message received");
                            let bytes = packet.bytes;
                            let base_header = BaseHeader::from_bytes(bytes.clone()).unwrap();
                            let message_packet = MessagePacket::from_bytes(BytesMut::from(
                                // Create BytesMut
                                bytes[BASE_HEADER_SIZE..] // Get remaining bytes (without header)
                                    .to_vec()
                                    .as_slice(), // Convert Vec<u8> to &[u8]
                            ));
                    
                            info!(
                                "MSG from {}:\n{}",
                                base_header.get_source(),
                                message_packet.get_message()
                            );
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Error while reading socket: {:?} - Dropping socket connection!",
                        e
                    );
                    // Remove entry from routing table
                    if !self.target_name.is_empty() {
                        self.morganite
                            .lock()
                            .await
                            .remove_entry(self.target_name.clone())
                            .await;
                    }
                    return;
                }
            }
        }
    }
}

impl Default for SocketReadHandler {
    fn default() -> Self {
        panic!("Default constructor not implemented");
    }
}
