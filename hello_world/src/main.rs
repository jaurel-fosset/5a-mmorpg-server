
use game_sockets::{GameConnection, GameNetworkEvent, GamePeer, GameStream, GameStreamReliability};
use std::time::{Duration, Instant};
use bytes::{Bytes, BytesMut};
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::{SubscribePacket};
use network_serialization::packets::Packet;
use network_serialization::packets::topic::{TopicLeaf, TopicNode, TopicTree, TopicTreeType};
use network_serialization::{Deserializable, Serializable};

fn main() {
    let mut tree_shard = TopicTree::new_empty("shard:0".to_string());

    // Player 42
    let mut tree_player42 = TopicTree::new_empty("player:42".to_string());

    let pos: Vec<u32> = vec![300, 200];
    let mut bytes = BytesMut::new();
    let _ = pos.serialize(&mut bytes);
    tree_player42.add_leaf("position".to_string(), Vec::<u8>::from(bytes));

    let health: Vec<u32> = vec![100];
    let mut bytes = BytesMut::new();
    let _ = health.serialize(&mut bytes);
    tree_player42.add_leaf("health".to_string(), Vec::<u8>::from(bytes));

    // Player 43
    let mut tree_player43 = TopicTree::new_empty("player:43".to_string());

    let pos: Vec<u32> = vec![150, 400];
    let mut bytes = BytesMut::new();
    let _ = pos.serialize(&mut bytes);
    tree_player43.add_leaf("position".to_string(), Vec::<u8>::from(bytes));

    let health: Vec<u32> = vec![80];
    let mut bytes = BytesMut::new();
    let _ = health.serialize(&mut bytes);
    tree_player43.add_leaf("health".to_string(), Vec::<u8>::from(bytes));

    tree_shard.add_tree(tree_player42);
    tree_shard.add_tree(tree_player43);

    // Test keys()
    println!("=== keys ===");
    for key in tree_shard.clone().keys() {
        println!("{}", String::from_utf8(key).unwrap());
    }

    // Test get_sub_tree()
    println!("\n=== get_sub_tree player:42 ===");
    println!("{:?}", tree_shard.get_sub_tree("shard:0/player:42"));

    println!("\n=== get_sub_tree * ===");
    if let Some(sub) = tree_shard.get_sub_tree("shard:0/*") {
        for (key, value) in sub.flatten() {
            println!("{} → {:?}", String::from_utf8(key).unwrap(), value);
        }
    }

    // Test get()
    println!("\n=== get position player:42 ===");
    println!("{:?}", tree_shard.get("shard:0/player:42/position"));


    let mut tree_a = TopicTree::new_empty("shard:0".to_string());
    let mut player42 = TopicTree::new_empty("player:42".to_string());
    player42.add_leaf("position".to_string(), vec![1, 2, 3]);
    tree_a.add_tree(player42);

    let mut tree_b = TopicTree::new_empty("shard:0".to_string());
    let mut player43 = TopicTree::new_empty("player:43".to_string());
    player43.add_leaf("position".to_string(), vec![4, 5, 6]);
    tree_b.add_tree(player43);

    tree_a.merge(&tree_b);

    println!("\n=== Test merge ===");
    println!("{:?}", tree_a.get_sub_tree("shard:0/player:43/*"));

    /*
    let mut treeShard = TopicTree::new_empty("shard:0".to_string());
    let mut playerTree = TopicTree::new_empty("player:42".to_string());

    let player_pos: Vec<u32> = vec!(300, 200);
    let mut bytes = BytesMut::new();
    let _ = player_pos.serialize(&mut bytes);
    let player_pos = Vec::<u8>::from(bytes);
    playerTree.add_leaf("position".to_string(), player_pos);

    let player_vel: Vec<u32> = vec!(100, 150);
    let mut bytes = BytesMut::new();
    let _ = player_vel.serialize(&mut bytes);
    let player_vel = Vec::<u8>::from(bytes);
    playerTree.add_leaf("velocity".to_string(), player_vel);

    treeShard.add_tree(playerTree);

    for key in treeShard.clone().keys() {
        println!("{:?}",String::from_utf8(key));
    }

    let packet = SubscribePacket{ client_id: 32u32, topic: treeShard.clone() };
    let mut buffer = BytesMut::new();
    packet.serialize(&mut buffer).unwrap();
    let mut bytes : Bytes = buffer.freeze();
    let packet = SubscribePacket::deserialize(&mut bytes);


    for key in packet.clone().unwrap().topic.clone().keys() {
        println!("{:?}",String::from_utf8(key));
    }

    println!("{:?}",treeShard.flatten());
    println!("{:?}",packet.unwrap().topic.clone().flatten());


    return;*/
    /*
    let backend = game_sockets::protocols::QuicBackend::new();
    let mut peer = GamePeer::new(backend);

    peer.connect("127.0.0.1", 10001).unwrap();

    // Étape 1 — Connected local
    let conn: GameConnection = loop {
        if let Ok(Some(GameNetworkEvent::Connected(conn))) = peer.poll() {
            println!("Connected! {:?}", conn);
            break conn;
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    // Étape 2 — Stream
    peer.create_stream(conn.clone(), GameStreamReliability::Unreliable).unwrap();
    let stream: GameStream = loop {
        if let Ok(Some(GameNetworkEvent::StreamCreated(_, stream))) = peer.poll() {
            break stream;
        }
        std::thread::sleep(Duration::from_millis(10));
    };


    let client_id = 50;
    let topic = [3u8; 32];
    let data = SubscribePacket{client_id, topic};
    let packet = PacketMessage::new(PacketData::Subscribe(data));

    // Étape 3 — Envoie
    peer.send(&conn, &stream, packet.write().unwrap()).unwrap();
    println!("Packet sent to 127.0.0.1:5555, waiting for response...");

    // Étape 4 — Attend une réponse
    let timeout = Instant::now();
    loop {
        if timeout.elapsed() > Duration::from_secs(15) {
            println!("Timeout — no response");
            break;
        }

        match peer.poll() {
            Ok(Some(GameNetworkEvent::Message { connection, stream, data })) => {
                println!("Response from {:?}: {:?}", connection, data);
            }
            Ok(Some(e)) => println!("Event: {:?}", e),
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }*/
}