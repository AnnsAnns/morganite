use bytes::{BufMut, BytesMut};
use log::{error, warn};
use num_enum::{IntoPrimitive, TryFromPrimitive};

/**
 * The BaseHeader struct
 */
pub struct BaseHeader {
    packet_type: PacketType,
    ttl: u8,
    target: String,
    source: String,
}

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PacketType {
    Connection,
    Routing,
    Message,
}

impl PacketType {
    pub fn from_u8(value: u8) -> Option<PacketType> {
        match PacketType::try_from(value) {
            Ok(packet_type) => Some(packet_type),
            Err(e) => {
                error!("Error while parsing packet type: {:?}", e);
                None
            }
        }
    }

    pub fn to_u8(&self) -> u8 {
        self.clone().into()
    }
}

pub const BASE_HEADER_SIZE: usize = 64 / 8;

impl BaseHeader {
    /**
     * Creates a new BaseHeader
     * @warning Truncates the target and source addresses to 3 characters
     */
    pub fn new(
        packet_type: PacketType,
        ttl: u8,
        target: String,
        source: String
    ) -> BaseHeader {
        let mut truncated_target = target.clone();
        let mut truncated_source = source.clone();

        if target.len() > 3 {
            warn!("Target address is too long, truncating to 3 characters");
            truncated_target.truncate(3);
        }

        if source.len() > 3 {
            warn!("Source address is too long, truncating to 3 characters");
            truncated_source.truncate(3);
        }

        BaseHeader {
            packet_type,
            ttl,
            target: truncated_target,
            source: truncated_source,
        }
    }

    /**
     * Returns the header as a BytesMut
     */
    pub fn to_bytes(&self) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(BASE_HEADER_SIZE);
        bytes.put_u8(self.packet_type.clone().into()); // Transform enum to u8
        bytes.put_u8(self.ttl);
        bytes.put(self.target.as_bytes());
        bytes.put(self.source.as_bytes());
        bytes
    }

    #[allow(dead_code)] // Will be used later
    pub fn get_ip(&self) -> String {
        self.source.clone()
    }

    pub fn get_target(&self) -> String {
        self.target.clone()
    }

    pub fn get_source(&self) -> String {
        self.source.clone()
    }

    /**
     * Creates a new BaseHeader from a BytesMut
     */
    pub fn from_bytes(bytes: BytesMut) -> Option<BaseHeader> {
        let packet_type = match PacketType::try_from(bytes[0]) {
            Ok(packet_type) => packet_type,
            Err(e) => {
                error!("Error while parsing packet type: {:?}", e);
                return None;
            }
        };
        let ttl = bytes[1];
        let target = String::from_utf8(bytes[2..5].to_vec()).unwrap();
        let source = String::from_utf8(bytes[5..8].to_vec()).unwrap();

        Some(BaseHeader {
            packet_type,
            ttl,
            target,
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_header() {
        let header = BaseHeader::new(
            PacketType::Connection,
            0,
            String::from("ABC"),
            String::from("DEF"),
        );
        let bytes = header.to_bytes();
        let header = BaseHeader::from_bytes(bytes).unwrap();

        assert_eq!(header.get_ip(), String::from("DEF"));
        assert_eq!(header.get_target(), String::from("ABC"));
        assert_eq!(header.get_source(), String::from("DEF"));
    }

    #[test]
    fn test_base_header_long() {
        let header = BaseHeader::new(
            PacketType::Connection,
            0,
            String::from("ABCDEF"),
            String::from("DEFGHI"),
        );
        let bytes = header.to_bytes();
        let header = BaseHeader::from_bytes(bytes).unwrap();

        assert_eq!(header.get_ip(), String::from("DEF"));
        assert_eq!(header.get_target(), String::from("ABC"));
        assert_eq!(header.get_source(), String::from("DEF"));
    }
}