use bytes::BytesMut;
use game_sockets::{
    GameConnection, GameNetworkEvent, GameStream, GameStreamReliability,
};
use network_serialization::Serializable;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::Packet;
use network_serialization::packets::broker::{PublishPacket};
use network_serialization::packets::topic::TopicTree;
use std::time::Duration;

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

    peer.create_stream(conn.clone(), GameStreamReliability::Unreliable)
        .unwrap();
    let stream: GameStream = loop {
        if let Ok(Some(GameNetworkEvent::StreamCreated(_, stream))) = peer.poll() {
            break stream;
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    loop {
        loop {
            match peer.poll() {
                Ok(Some(GameNetworkEvent::Message { data, .. })) => {
                    let msg = PacketMessage::read(data).unwrap();
                    match msg.data {
                        PacketData::Broadcast(packet) => {
                            for tree in packet.data {
                                let flat = tree.flatten();
                                for (key, value) in flat {
                                    println!("Reçu: {} → {:?}", String::from_utf8(key).unwrap(), value);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(None) => {break;}
                _ => {}
            }
        }


        let mut tree_entities = TopicTree::new_empty("entities".to_string());
        // Position
        let mut tree_position = TopicTree::new_empty("position".to_string());

        let pos: Vec<i32> = vec![300, 200];
        let mut bytes = BytesMut::new();
        let _ = pos.serialize(&mut bytes);
        tree_position.add_leaf("2".to_string(), Vec::<u8>::from(bytes));

        let pos: Vec<i32> = vec![100, 200];
        let mut bytes = BytesMut::new();
        let _ = pos.serialize(&mut bytes);
        tree_position.add_leaf("3".to_string(), Vec::<u8>::from(bytes));

        let pos: Vec<i32> = vec![-50, 142];
        let mut bytes = BytesMut::new();
        let _ = pos.serialize(&mut bytes);
        tree_position.add_leaf("4".to_string(), Vec::<u8>::from(bytes));

        tree_entities.add_tree(tree_position);

        // Velocity
        let mut tree_velocity = TopicTree::new_empty("velocity".to_string());

        let pos: Vec<i32> = vec![150, 400];
        let mut bytes = BytesMut::new();
        let _ = pos.serialize(&mut bytes);
        tree_velocity.add_leaf("2".to_string(), Vec::<u8>::from(bytes));

        let pos: Vec<i32> = vec![0, -500];
        let mut bytes = BytesMut::new();
        let _ = pos.serialize(&mut bytes);
        tree_velocity.add_leaf("3".to_string(), Vec::<u8>::from(bytes));

        tree_entities.add_tree(tree_velocity);

        /*
        if let Some(sub) = tree_entities.get_sub_tree("shard:0/\*") {
            for (key, value) in sub.flatten() {
                println!("{} → {:?}", String::from_utf8(key).unwrap(), value);
            }
        }
         */

        let packet = PacketMessage::new(PacketData::Publish(PublishPacket {
            data: vec![tree_entities],
        }));

        let bytes = packet.write().unwrap();

        peer.send(&conn, &stream, bytes).unwrap();
        println!("Packet sent");

        std::thread::sleep(Duration::from_secs(1));
    }
}
