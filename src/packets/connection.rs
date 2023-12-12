use bytes::{BufMut, BytesMut};

pub struct ConnectionPacket {
    pub name: String,
    pub port: u16,
    pub is_first: bool,
}

impl ConnectionPacket {
    pub fn new(name: String, port: u16, is_first: bool) -> ConnectionPacket {
        ConnectionPacket { name, port, is_first }
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(1024);
        bytes.put(self.name.as_bytes());
        bytes.put_u16(self.port);
        bytes.put_u8(self.is_first as u8);
        bytes
    }

    pub fn from_bytes(bytes: BytesMut) -> ConnectionPacket {
        let name = String::from_utf8(bytes[0..3].to_vec()).unwrap();
        let port = u16::from_be_bytes([bytes[3], bytes[4]]);
        let is_first = bytes[5] == 1;

        ConnectionPacket { name, port, is_first }
    }
}
