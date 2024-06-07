use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SharedHeader {
    pub source_ip: String,
    pub source_port: String,
    pub destination_ip: String,
    pub destination_port: String,
    pub ttl: u8,
}