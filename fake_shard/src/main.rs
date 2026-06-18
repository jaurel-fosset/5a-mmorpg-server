use std::collections::HashMap;
use bytes::{Bytes, BytesMut};
use game_sockets::{
    GameConnection, GameNetworkEvent, GameStream, GameStreamReliability,
};
use network_serialization::{Deserializable, Serializable};
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::Packet;
use network_serialization::packets::broker::{ClientHelloPacket, NetworkId, PublishPacket, SubscribePacket};
use network_serialization::packets::topic::{TopicTree, TopicTreeType};
use std::time::{Duration, Instant};

#[derive(Default)]
struct ShardState {
    data_position: HashMap<u32, [f32;2]>,
    last_input_sequence: HashMap<u32, u32>,
}

fn main() {
    println!("Hello, world!");

    let mut shard_state = ShardState::default();

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

    let bytes = PacketMessage::new(
        PacketData::ClientHello(
            ClientHelloPacket{
                client_type: NetworkId::Shard
            }
        )
    ).write().unwrap();

    peer.send(&conn,&stream,bytes).unwrap();
    std::thread::sleep(Duration::from_millis(100));

    let mut tree_entities = TopicTree::new_empty("entities".to_string());
    let mut tree_input = TopicTree::new_empty("input".to_string());
    tree_input.add_leaf("*".to_string(),Vec::new());
    tree_entities.add_tree(tree_input);

    let bytes = PacketMessage::new(
        PacketData::Subscribe(
            SubscribePacket{
                client_id: 2,
                topic: tree_entities,
            }
        )
    ).write().unwrap();

    peer.send(&conn,&stream,bytes).unwrap();
    std::thread::sleep(Duration::from_millis(100));


    let tick_duration = Duration::from_millis(66);

    loop {
        let start_time = Instant::now();
        loop {
            match peer.poll() {
                Ok(Some(GameNetworkEvent::Message { data, .. })) => {
                    let msg = PacketMessage::read(data).unwrap();
                    match msg.data {
                        PacketData::Broadcast(packet) => for tree in packet.data {

                            // On récupère les inputs
                            let Some(input_tree) = tree.get_child("input") else { break; };
                            let TopicTreeType::Node(input_topic_node) = &input_tree.item else {break;};
                            for topic_tree in &input_topic_node.data {
                                let TopicTreeType::Leaf(leaf) = &topic_tree.item else {continue;};
                                println!("topic_tree.name: {:?}", topic_tree.name);
                                let Ok(client_id) = topic_tree.name.parse::<u32>() else {continue;};
                                let data = &leaf.data;
                                let mut bytes : Bytes = Bytes::copy_from_slice(data);
                                let Ok(inputs) = <[InputData;16]>::deserialize(&mut bytes) else {continue;};
                                move_entity(&mut shard_state, client_id, inputs);
                            }
                        },
                        _ => {}
                    }
                }
                Ok(None) => {break;}
                _ => {}
            }
        }


        let tree_entities = build_entities_tree(&shard_state);
        println!("{:?}", tree_entities);

        let packet = PacketMessage::new(PacketData::Publish(PublishPacket {
            data: vec![tree_entities],
        }));
        let bytes = packet.write().unwrap();

        peer.send(&conn, &stream, bytes).unwrap();

        // Sleep seulement le temps restant
        let work_duration = start_time.elapsed();
        if let Some(sleep_duration) = tick_duration.checked_sub(work_duration) {
            std::thread::sleep(sleep_duration);
        } else {
            println!("LAG: work took {}ms", work_duration.as_millis());
        }
    }
}

use network_serialization::input::{DirectionFlags, InputData};

fn move_entity(
    shard_state: &mut ShardState,
    client_id: u32,
    inputs: [InputData; 16],
) {
    let position = shard_state.data_position.entry(client_id).or_insert([20f32, 20f32]);
    let last_sequence = shard_state.last_input_sequence.entry(client_id).or_insert(0);

    //println!("Position: {:?}, Input: {:?}", position, last_sequence);

    for input in inputs {
        if input.sequence > *last_sequence {
            *last_sequence = input.sequence;

            if input.input.is_empty() {continue;}

            if input.input.contains(DirectionFlags::UP) {
                position[0] += 10f32 ;
            }
            if input.input.contains(DirectionFlags::DOWN) {
                position[0] -= 10f32 ;
            }
            if input.input.contains(DirectionFlags::LEFT) {
                position[1] -= 10f32 ;
            }
            if input.input.contains(DirectionFlags::RIGHT) {
                position[1] += 10f32 ;
            }
        }
    }
}

fn build_entities_tree(shard_state: &ShardState) -> TopicTree {
    let mut tree_entities = TopicTree::new_empty("entities".to_string());
    // Position
    let mut tree_position = TopicTree::new_empty("position".to_string());
    for (key, value) in shard_state.data_position.iter() {
        let mut bytes = BytesMut::new();
        let _ = value[0].serialize(&mut bytes);
        let _ = value[1].serialize(&mut bytes);
        tree_position.add_leaf(key.to_string(), Vec::<u8>::from(bytes));
    }

    tree_entities.add_tree(tree_position);
    tree_entities
}