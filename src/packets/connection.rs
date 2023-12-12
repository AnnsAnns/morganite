use bytes::{BufMut, BytesMut};

pub struct ConnectionPacket {
    pub name: String,
    pub port: u16,
    pub isFirst: bool,
}

impl ConnectionPacket {
    pub fn new(name: String, port: u16, isFirst: bool) -> ConnectionPacket {
        ConnectionPacket { name, port, isFirst }
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(1024);
        bytes.put(self.name.as_bytes());
        bytes.put_u16(self.port);
        bytes.put_u8(self.isFirst as u8);
        bytes
    }

    pub fn from_bytes(bytes: BytesMut) -> ConnectionPacket {
        let name = String::from_utf8(bytes[0..3].to_vec()).unwrap();
        let port = u16::from_be_bytes([bytes[3], bytes[4]]);
        let isFirst = bytes[5] == 1;

        ConnectionPacket { name, port, isFirst }
    }
}
