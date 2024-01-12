pub mod socket;
pub mod socket_read_handler;

use crate::listener::socket::SocketStream;
use crate::Morganite;
use tokio_task_pool::Pool;

use colored::Colorize;
use log::{debug, warn};

use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;

const TASK_POOL_SIZE: usize = 20;

pub struct Listener {
    morganite: Arc<Mutex<Morganite>>,
    listener: TcpListener,
    task_pool: Pool,
    own_name: String,
}

impl Listener {
    pub async fn new(morganite: Arc<Mutex<Morganite>>, listener: TcpListener, own_name: String) -> Listener {
        Listener {
            morganite,
            listener,
            task_pool: Pool::bounded(TASK_POOL_SIZE),
            own_name,
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
                        self.own_name.clone(),
                    );
                    self.task_pool.spawn(async move {
                        handler.process().await;
                    }).await.unwrap();
                }
                Err(e) => {
                    warn!("Client connection failed = {:?}", e);
                }
            }
        }
    }
}
