use bytes::{BufMut, BytesMut};
use log::{warn};

/**
 * The BaseHeader struct
 */
pub struct BaseHeader {
    packet_type: u8,
    ttl: u8,
    target: String,
    source: String,
    hops: u8,
}

impl BaseHeader {
    /**
     * Creates a new BaseHeader
     * @warning Truncates the target and source addresses to 3 characters
     */
    pub fn new(packet_type: u8, ttl: u8, target: String, source: String, hops: u8) -> BaseHeader {
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
            hops,
        }
    }

    /**
     * Returns the header as a BytesMut
     */
    pub fn to_bytes(&self) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(72/8);
        bytes.put_u8(self.packet_type);
        bytes.put_u8(self.ttl);
        bytes.put(self.target.as_bytes());
        bytes.put(self.source.as_bytes());
        bytes.put_u8(self.hops);
        bytes
    }
}