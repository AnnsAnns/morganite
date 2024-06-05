use serde::{Deserialize, Serialize};

pub const COMMON_HEADER_LENGTH: usize = 50;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct CommonHeader {
    pub length: u16,
    pub crc32: u32,
    pub type_id: u8,
}