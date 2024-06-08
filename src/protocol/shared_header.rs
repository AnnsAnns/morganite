use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SharedHeader {
    pub source_ip: String,
    pub source_port: String,
    pub dest_ip: String,
    pub dest_port: String,
    pub ttl: u8,
}