use crate::{Morganite};
use bytes::{BytesMut};
use log::{debug, info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::{io};
use tokio::io::{AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};

pub struct ConnectionHandler {
    morganite: Arc<Mutex<Morganite>>,
    listener: TcpListener,
}

impl ConnectionHandler {
    pub async fn new(morganite: Arc<Mutex<Morganite>>, listener: TcpListener) -> ConnectionHandler {
        ConnectionHandler {
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
        let mut msg = BytesMut::with_capacity(1024);

        loop {
            socket.readable().await.unwrap();

            match socket.read(&mut msg).await {
                Ok(n) => {
                    if n == 0 {
                        return;
                    }
                    debug!("Received {} bytes", n);
                    debug!("Received: {:?}", msg);
                    msg.clear();
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    println!("Error while reading socket: {:?}", e);
                    return;
                }
            }
        }
    }
}
