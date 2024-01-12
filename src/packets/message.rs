

use bytes::{BufMut, BytesMut};


pub struct MessagePacket {
    pub msg: String,
}

impl MessagePacket {
    #[allow(dead_code)] // Will be used later
    pub fn new(msg: String) -> MessagePacket {
        MessagePacket { msg }
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(320);
        bytes.put(self.msg.as_bytes());
        // Fill rest of bytes with 0 because our protocol asks for exactly 320 bytes
        // (Kinda stupid but whatever, we can't change it now)
        for _ in 0..(320 - self.msg.len()) {
            bytes.put_u8(0);
        }
        bytes
    }

    pub fn get_message(&self) -> String {
        self.msg.clone()
    }

    pub fn from_bytes(bytes: BytesMut) -> MessagePacket {
        let msg = String::from_utf8(bytes[0..bytes.len()].to_vec()).unwrap().trim_end().to_string();

        // Remove leading and trailing 0 bytes
        let msg = msg.trim_start_matches(char::from(0)).to_string();
        let msg = msg.trim_end_matches(char::from(0)).to_string();

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