use crate::packets::header::PacketType;
use crate::packets::Packet;
use crate::Morganite;

use log::{debug, error};

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct SocketHandler {
    morganite: Arc<Mutex<Morganite>>,
}

impl SocketHandler {
    pub fn new(morganite: Arc<Mutex<Morganite>>) -> SocketHandler {
        SocketHandler { morganite }
    }

    pub async fn process(&mut self, socket: &mut TcpStream) {
        loop {
            let mut msg = Vec::new();

            match socket.read_to_end(&mut msg).await {
                Ok(n) => {
                    if n == 0 {
                        return;
                    }
                    debug!("Received {} bytes", n);
                    debug!("Received: {:?}", msg);
                    // get first byte to determine type of message
                    let msg_type = match PacketType::from_u8(msg[0].clone()) {
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

                    match msg_type {
                        PacketType::Connection => {
                            // Connection message
                            //@TODO
                            debug!("Connection message received");
                        }
                        PacketType::Routing => {
                            // Routing message
                            debug!("Routing message received");
                            self.morganite
                                .lock()
                                .await
                                .update_routing_table(
                                    packet.bytes,
                                    socket.peer_addr().unwrap().to_string(),
                                )
                                .await;
                        }
                        PacketType::Message => {
                            // Data message
                            //@TODO
                            debug!("Data message received");
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "Error while reading socket: {:?} - Dropping socket connection!",
                        e
                    );
                    socket.shutdown().await.unwrap();
                    return;
                }
            }
        }
    }
}

impl Default for SocketHandler {
    fn default() -> Self {
        panic!("Default constructor not implemented");
    }
}
