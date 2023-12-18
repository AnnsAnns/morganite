pub mod socket_read_handler;
pub mod socket_write_handler;
pub mod socket;

use crate::Morganite;
use crate::listener::socket::SocketStream;

use log::{info, warn};

use std::sync::Arc;

use tokio::net::TcpListener;
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
        info!("Listening for new connections at {}", self.listener.local_addr().unwrap());

        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    info!("New client connection {:?}", addr);

                    let socket = Arc::new(Mutex::new(SocketStream::new(socket)));

                    let mut handler = socket_read_handler::SocketReadHandler::new(self.morganite.clone(), socket);
                    tokio::spawn(async move {
                        handler.process().await;
                    });
                }
                Err(e) => {
                    warn!("Client connection failed = {:?}", e);
                }
            }
        }
    }
}
