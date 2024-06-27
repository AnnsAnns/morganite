use serde::{Deserialize, Serialize};

use super::shared_header::SharedHeader;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoutedPacket {
    pub header: SharedHeader,
    pub nickname: String,
    pub message: String,
}

#[test]
fn test_parsing_routed_packet() {
    let json = r#"{"header":{
        "packet_type":       "data_message",
        "source_ip":          "192.168.123.122",
        "source_port":        6827,
        "dest_ip":     "192.168.234.233",
        "dest_port":   234,
        "ttl":                16},
        "nickname":           "Test",
        "message":               "Test Data"
    }"#;

    let packet: RoutedPacket = serde_json::from_str(json).unwrap();
    assert_eq!(packet.header.source_ip, "192.168.123.122");
    assert_eq!(packet.header.source_port, 6827);
    assert_eq!(packet.header.dest_ip, "192.168.234.233");
    assert_eq!(packet.header.dest_port, 234);
    assert_eq!(packet.nickname, "Test");
    assert_eq!(packet.header.ttl, 16);
    assert_eq!(packet.message, "Test Data");
}

#[test]
fn test_serializing_routed_packet() {
    let packet = RoutedPacket {
        header: SharedHeader {
            source_ip: "192.168.101.101".to_string(),
            source_port: 1234,
            dest_ip: "153.132.143.121".to_string(),
            dest_port: 4321,
            ttl: 32,
        },
        nickname: "Test".to_string(),
        message: "Testing".to_string(),
    };

    let json = serde_json::to_string(&packet).unwrap();

    assert_eq!(
        json,
        r#"{"header":{"source_ip":"192.168.101.101","source_port":1234,"dest_ip":"153.132.143.121","dest_port":4321,"ttl":32},"nickname":"Test","message":"Testing"}"#
    );
}
