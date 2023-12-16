use bytes::{BufMut, BytesMut};

pub mod connection;
pub mod header;
pub mod routing_entry;
pub mod message;

pub struct Packet {
    pub bytes: BytesMut,
    pub checksum: u32,
}

impl Packet {
    pub fn to_bytes(&self) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(1024);
        bytes.put(self.bytes.clone());
        bytes.put_u32(self.checksum);
        bytes
    }

    pub fn from_bytes(mut bytes: Vec<u8>) -> Packet {
        // Check if empty
        if bytes.len() == 0 {
            return Packet {
                bytes: BytesMut::new(),
                checksum: 0,
            };
        }

        let checksum_vec = bytes.split_off(bytes.len() - 4); // Split off last 32bit into checksum
        let bytes = BytesMut::from(bytes.as_slice()); // Convert rest of message into BytesMut

        let checksum = u32::from_be_bytes([
            checksum_vec[0],
            checksum_vec[1],
            checksum_vec[2],
            checksum_vec[3],
        ]);

        Packet { bytes, checksum }
    }

    /**
     * Creates a new CRC32 checksum from the given bytes
     */
    pub async fn create_crc32(bytes: BytesMut) -> Packet {
        let mut crc32 = crc32fast::Hasher::new();
        crc32.update(&bytes);

        Packet {
            bytes,
            checksum: crc32.finalize(),
        }
    }

    /**
     * Verifies the given CRC32 checksum against the given bytes
     */
    pub async fn verify_crc32(bytes: BytesMut, checksum: u32) -> bool {
        let mut crc32 = crc32fast::Hasher::new();
        crc32.update(&bytes);
        crc32.finalize() == checksum
    }

    /**
     * Verifies the checksum of the packet
     */
    pub async fn verify_self(&self) -> bool {
        Packet::verify_crc32(self.bytes.clone(), self.checksum).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet() {
        let bytes = BytesMut::from("Hello World!".as_bytes());
        let packet = Packet::from_bytes(bytes.clone().to_vec());
        let bytes = packet.to_bytes();

        assert_eq!(bytes, BytesMut::from("Hello World!".as_bytes()));
    }

    #[test]
    fn test_packet_long() {
        let bytes = BytesMut::from("Hello World! This is a very long message that should be able to be sent over the network. I hope this works!".as_bytes());
        let packet = Packet::from_bytes(bytes.clone().to_vec());
        let bytes = packet.to_bytes();

        assert_eq!(bytes, BytesMut::from("Hello World! This is a very long message that should be able to be sent over the network. I hope this works!".as_bytes()));
    }
}