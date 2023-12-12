use std::fmt::{self, Display, Formatter};

use bytes::{BufMut, BytesMut};
use log::warn;

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

    pub fn from_bytes(bytes: BytesMut) -> MessagePacket {
        let msg = String::from_utf8(bytes[0..bytes.len()].to_vec()).unwrap();

        MessagePacket { msg }
    }
}