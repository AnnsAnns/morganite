use crate::Morganite;
use bytes::BytesMut;
use log::{debug, info, warn};

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub struct Listener {
    morganite: Arc<Mutex<Morganite>>,
    listener: TcpListener,
}

impl Listener {
    pub async fn new(morganite: Arc<Mutex<Morganite>>, listener: TcpListener) -> Listener {
        Listener {
            morganite,
            listener,
        }
    }

    pub async fn listen(&mut self) {
        info!("Listening for new connections!");

        loop {
            match self.listener.accept().await {
                Ok((mut socket, addr)) => {
                    info!("New client connection {:?}", addr);
                    self.process(&mut socket).await;
                }
                Err(e) => {
                    warn!("Client connection failed = {:?}", e);
                }
            }
        }
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
                let msg_type = msg[0];
                match msg_type {
                    0 => {
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
                    1 => {
                        // Data message
                        //@TODO
                    }
                    _ => {
                        warn!("Unknown message type: {}", msg_type);
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
