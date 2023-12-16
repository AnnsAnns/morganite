use std::{sync::Arc};

use log::debug;
use tokio::{net::TcpStream, io::AsyncWriteExt};

use crate::{morganite::Morganite, packets::Packet};

pub struct SocketWriteHandler {
    socket: TcpStream,
    pub addr: String,
}

impl SocketWriteHandler {
    pub async fn new(mut socket: TcpStream, addr: String) -> SocketWriteHandler {
        debug!("New SocketWriteHandler for {}", addr);

        socket.write(b"Test").await.unwrap();

        SocketWriteHandler {
            socket,
            addr,
        }
    }

    pub async fn write(&mut self, msg: Packet) {
        debug!("Sending packet {:?} to {}", msg, self.addr);
        let mut msg = msg.to_bytes();
        self.socket.writable().await.unwrap();
        self.socket.write_all(&mut msg).await.unwrap();
        self.socket.flush().await.unwrap();
    }
}