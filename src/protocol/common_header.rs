use serde::{Deserialize, Serialize};

pub const COMMON_HEADER_LENGTH: usize = 53;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct CommonHeader {
    pub length: u16,
    pub crc32: u32,
    pub type_id: u8,
}

impl CommonHeader {
    pub fn from_unparsed(header: CommonHeaderUnparsed) -> Self {
        Self {
            length: header.length.parse().unwrap(),
            crc32: header.crc32.parse().unwrap(),
            type_id: header.type_id.parse().unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommonHeaderUnparsed {
    pub length: String,
    pub crc32: String,
    pub type_id: String,
}

impl CommonHeaderUnparsed {
    pub fn new(header: CommonHeader) -> Self {
        Self {
            length: format!("{:0>5}", header.length),
            crc32: format!("{:0>10}", header.crc32),
            type_id: format!("{:0>1}", header.type_id),
        }
    }
}
