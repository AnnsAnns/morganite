use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoutedPacket {
    packet_type: String,
    source_ip: String,
    source_port: String,
    destination_ip: String,
    destination_port: String,
    ttl: String,
    nickname: String,
    data: String,
}

#[test]
fn test_parsing_routed_packet() {
    let json = r#"{
        "packet_type":       "data_message",
        "source_ip":          "192.168.123.122",
        "source_port":        "6827",
        "destination_ip":     "192.168.234.233",
        "destination_port":   "234",
        "nickname":           "Test",
        "ttl":                "16",
        "data":               "Test Data"
    }"#;

    let packet: RoutedPacket = serde_json::from_str(json).unwrap();
    assert_eq!(packet.packet_type, "data_message");
    assert_eq!(packet.source_ip, "192.168.123.122");
    assert_eq!(packet.source_port, "6827");
    assert_eq!(packet.destination_ip, "192.168.234.233");
    assert_eq!(packet.destination_port, "234");
    assert_eq!(packet.nickname, "Test");
    assert_eq!(packet.ttl, "16");
    assert_eq!(packet.data, "Test Data");
}

#[test]
fn test_serializing_routed_packet() {
    let packet = RoutedPacket {
        packet_type: "TESTTYPE".to_string(),
        source_ip: "192.168.101.101".to_string(),
        source_port: "1234".to_string(),
        destination_ip: "153.132.143.121".to_string(),
        destination_port: "4321".to_string(),
        nickname: "Test".to_string(),
        ttl: "32".to_string(),
        data: "Testing".to_string(),
    };

    let json = serde_json::to_string(&packet).unwrap();

    assert_eq!(
        json,
        r#"{"packet_type":"TESTTYPE","source_ip":"192.168.101.101","source_port":"1234","destination_ip":"153.132.143.121","destination_port":"4321","ttl":"32","nickname":"Test","data":"Testing"}"#
    );
}
