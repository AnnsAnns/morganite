pub mod socket;
pub mod socket_read_handler;

use crate::listener::socket::SocketStream;
use crate::Morganite;

use colored::Colorize;
use log::{debug, warn};

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
        println!(
            "{} {} {}",
            "(`Д´)ゞ".bold().red(),
            "Listening for new connections at".red().italic(),
            self.listener
                .local_addr()
                .unwrap()
                .to_string()
                .bright_red()
                .bold()
                .underline()
        );

        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    debug!(
                        "New client connection {:?}",
                        addr.to_string().bold().underline()
                    );

                    let own_addr = self.listener.local_addr().unwrap().to_string();
                    let socket = Arc::new(Mutex::new(SocketStream::new(socket)));

                    let mut handler = socket_read_handler::SocketReadHandler::new(
                        self.morganite.clone(),
                        socket,
                        addr.to_string(),
                        own_addr,
                    );
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
