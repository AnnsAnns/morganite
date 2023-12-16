

use bytes::{BufMut, BytesMut};


pub struct MessagePacket {
    pub msg: String,
}

impl MessagePacket {
    pub fn new(msg: String) -> MessagePacket {
        MessagePacket { msg }
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(320);
        bytes.put(self.msg.as_bytes());
        bytes
    }

    pub fn get_message(&self) -> String {
        self.msg.clone()
    }

    pub fn from_bytes(bytes: BytesMut) -> MessagePacket {
        let msg = String::from_utf8(bytes[0..bytes.len()].to_vec()).unwrap();

        MessagePacket { msg }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_packet() {
        let msg = String::from("Hello World!");
        let packet = MessagePacket::new(msg.clone());
        let bytes = packet.to_bytes();
        let packet = MessagePacket::from_bytes(bytes);

        assert_eq!(packet.get_message(), msg);
    }

    #[test]
    fn test_message_packet_long() {
        let msg = String::from("Hello World! This is a very long message that should be able to be sent over the network. I hope this works!");
        let packet = MessagePacket::new(msg.clone());
        let bytes = packet.to_bytes();
        let packet = MessagePacket::from_bytes(bytes);

        assert_eq!(packet.get_message(), msg);
    }

    #[test]
    fn test_message_packet_empty() {
        let msg = String::from("");
        let packet = MessagePacket::new(msg.clone());
        let bytes = packet.to_bytes();
        let packet = MessagePacket::from_bytes(bytes);

        assert_eq!(packet.get_message(), msg);
    }
}