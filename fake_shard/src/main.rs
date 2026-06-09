use game_sockets::{GameConnection, GameNetworkEvent, GameStream, GameStreamReliability};
use std::time::Duration;
use bytes::BytesMut;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::{ClientHelloPacket, PublishPacket};
use network_serialization::packets::Packet;
use network_serialization::packets::topic::TopicTree;
use network_serialization::Serializable;

fn main() {
    println!("Hello, world!");

    let backend = game_sockets::protocols::QuicBackend::new();
    let mut peer = game_sockets::GamePeer::new(backend);

    peer.connect("127.0.0.1", 10001).unwrap();

    let conn: GameConnection = loop {
        if let Ok(Some(GameNetworkEvent::Connected(conn))) = peer.poll() {
            println!("Connected! {:?}", conn);
            break conn;
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    peer.create_stream(conn.clone(), GameStreamReliability::Unreliable).unwrap();
    let stream: GameStream = loop {
        if let Ok(Some(GameNetworkEvent::StreamCreated(_, stream))) = peer.poll() {
            break stream;
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    loop {
        let mut tree_shard = TopicTree::new_empty("shard:0".to_string());
        let mut tree_player42 = TopicTree::new_empty("player:42".to_string());

        let pos: Vec<i32> = vec![300, 200];
        let mut bytes = BytesMut::new();
        let _ = pos.serialize(&mut bytes);
        tree_player42.add_leaf("position".to_string(), Vec::<u8>::from(bytes));

        let pos: Vec<i32> = vec![10, -10];
        let mut bytes = BytesMut::new();
        let _ = pos.serialize(&mut bytes);
        tree_player42.add_leaf("velocity".to_string(), Vec::<u8>::from(bytes));

        let health: Vec<u32> = vec![100];
        let mut bytes = BytesMut::new();
        let _ = health.serialize(&mut bytes);
        tree_player42.add_leaf("health".to_string(), Vec::<u8>::from(bytes));

        // Player 43
        let mut tree_player43 = TopicTree::new_empty("player:43".to_string());

        let pos: Vec<i32> = vec![150, 400];
        let mut bytes = BytesMut::new();
        let _ = pos.serialize(&mut bytes);
        tree_player43.add_leaf("position".to_string(), Vec::<u8>::from(bytes));


        let pos: Vec<i32> = vec![0, -500];
        let mut bytes = BytesMut::new();
        let _ = pos.serialize(&mut bytes);
        tree_player43.add_leaf("velocity".to_string(), Vec::<u8>::from(bytes));

        let health: Vec<u32> = vec![80];
        let mut bytes = BytesMut::new();
        let _ = health.serialize(&mut bytes);
        tree_player43.add_leaf("health".to_string(), Vec::<u8>::from(bytes));

        tree_shard.add_tree(tree_player42);
        tree_shard.add_tree(tree_player43);

        /*
        if let Some(sub) = tree_shard.get_sub_tree("shard:0/\*") {
            for (key, value) in sub.flatten() {
                println!("{} → {:?}", String::from_utf8(key).unwrap(), value);
            }
        }
         */

        let packet = PacketMessage::new(
            PacketData::Publish(
                PublishPacket{
                    topic: tree_shard,
                }
            )
        );

        let bytes = packet.write().unwrap();
        
        peer.send(&conn, &stream, bytes).unwrap();
        println!("Packet sent");

        std::thread::sleep(Duration::from_secs(1));
    }
}
