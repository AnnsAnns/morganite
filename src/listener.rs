pub mod socket_handler;

use crate::Morganite;

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
        info!("Listening for new connections!");

        loop {
            match self.listener.accept().await {
                Ok((mut socket, addr)) => {
                    info!("New client connection {:?}", addr);
                    let mut handler = socket_handler::SocketHandler::new(self.morganite.clone(), socket);
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
