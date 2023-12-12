use bytes::{BytesMut, BufMut};

pub struct ConnectionPacket {
    pub name: String,
}

impl ConnectionPacket {
    pub fn new(name: String) -> ConnectionPacket {
        ConnectionPacket { name }
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(1024);
        bytes.put(self.name.as_bytes());
        bytes
    }

    pub fn from_bytes(bytes: BytesMut) -> ConnectionPacket {
        let name = String::from_utf8(bytes[0..3].to_vec()).unwrap();

        ConnectionPacket { name }
    }

}