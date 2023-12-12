use crate::packets::header::PacketType;
use crate::{Morganite};

use bytes::BytesMut;
use log::{debug, warn, error};

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream};
use tokio::sync::Mutex;

pub struct SocketHandler {
    morganite: Arc<Mutex<Morganite>>,
}

impl SocketHandler {
    pub fn new(morganite: Arc<Mutex<Morganite>>) -> SocketHandler {
        SocketHandler { morganite }
    }

    pub async fn process(&mut self, socket: &mut TcpStream) {
        let mut msg = Vec::new();

        match socket.read_to_end(&mut msg).await {
            Ok(n) => {
                if n == 0 {
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

                match msg_type {
                    PacketType::Connection => {
                        // Connection message
                        //@TODO
                    }
                    PacketType::Routing => {
                        // Routing message
                        self.morganite
                            .lock()
                            .await
                            .update_routing_table(
                                BytesMut::from(msg.as_slice()),
                                socket.peer_addr().unwrap().to_string(),
                            )
                            .await;
                    }
                    PacketType::Message => {
                        // Data message
                        //@TODO
                    }
                    _ => {
                        warn!("Unknown message type: {:#?}", msg_type);
                    }
                }
            }
            Err(e) => {
                println!("Error while reading socket: {:?}", e);
                return;
            }
        }
        socket.shutdown().await.unwrap();
    }
}

impl Default for SocketHandler {
    fn default() -> Self {
        panic!("Default constructor not implemented");
    }
}