use std::sync::Arc;

use tokio::{net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}}, io::{AsyncWriteExt, AsyncReadExt}};

pub struct SocketStream {
    pub reader: OwnedReadHalf,
    pub writer: OwnedWriteHalf,
}

impl SocketStream {
    pub fn new(socket: TcpStream) -> SocketStream {
        let (mut reader, mut writer) = socket.into_split();
        SocketStream {
            reader,
            writer,
        }
    }

    pub async fn write(&mut self, msg: Vec<u8>) {
        self.writer.writable().await.unwrap();
        self.writer.write_all(&msg).await.unwrap();
        self.writer.flush().await.unwrap();
    }

    pub async fn read_to_end(&mut self, msg: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        self.reader.read_to_end(msg).await
    }
}