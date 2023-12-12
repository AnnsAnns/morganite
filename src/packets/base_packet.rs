use bytes::{BytesMut, BufMut};

use super::{header::{BaseHeader, BASE_HEADER_SIZE}, Packet};

pub struct BaseParsedPacket {
    pub header: BaseHeader,
    pub payload: BytesMut,
    pub checksum: u32,
}

impl BaseParsedPacket {
    pub fn new(header: BaseHeader, payload: BytesMut, checksum: u32) -> BaseParsedPacket {
        BaseParsedPacket {
            header,
            payload,
            checksum,
        }
    }

    pub fn from_packet(packet: Packet) -> BaseParsedPacket {
        let checksum = packet.checksum;
        let mut bytes = packet.bytes;

        let header = BaseHeader::from_bytes(bytes.clone()).unwrap();
        let payload = bytes.split_off(BASE_HEADER_SIZE); // Split off header

        BaseParsedPacket {
            header,
            payload,
            checksum,
        }
    }
}