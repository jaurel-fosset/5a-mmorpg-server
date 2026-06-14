use std::collections::HashMap;
use bytes::{Bytes, BytesMut};
use game_sockets::{
    GameConnection, GameNetworkEvent, GameStream, GameStreamReliability,
};
use network_serialization::{Deserializable, Serializable};
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::Packet;
use network_serialization::packets::broker::{PublishPacket};
use network_serialization::packets::topic::TopicTree;
use std::time::{Duration, Instant};

#[derive(Default)]
struct ShardState {
    data_position: HashMap<u32, [i32;2]>,
    last_input_sequence: HashMap<u32, u32>,
}

fn main() {
    println!("Hello, world!");

    let mut shard_state = ShardState::default();
    let data_position = &mut shard_state.data_position;
    data_position.insert(2,[100,200]);
    data_position.insert(3,[300,500]);
    data_position.insert(4,[400,500]);

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

    let tick_duration = Duration::from_millis(66);

    loop {
        let start_time = Instant::now();
        loop {
            match peer.poll() {
                Ok(Some(GameNetworkEvent::Message { data, .. })) => {
                    let msg = PacketMessage::read(data).unwrap();
                    match msg.data {
                        PacketData::Broadcast(packet) => for tree in packet.data {
                            let Some(data) = tree.get("entities/input/2") else {break;};
                            let mut bytes : Bytes = Bytes::copy_from_slice(data);
                            let Ok(inputs) = <[InputData;16]>::deserialize(&mut bytes) else {break;};

                            move_entity(&mut shard_state, 2u32, inputs);
                        },
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

        for (key, value) in shard_state.data_position.clone() {
            let mut bytes = BytesMut::new();
            let _ = value.serialize(&mut bytes);
            tree_position.add_leaf(key.to_string(), Vec::<u8>::from(bytes));
        }

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
    u: u32,
    inputs: [InputData; 16],
) {
    let Some(position) = shard_state.data_position.get_mut(&u) else {return;};
    let last_sequence = shard_state.last_input_sequence.entry(u).or_insert(0);

    //println!("Position: {:?}, Input: {:?}", position, last_sequence);

    for input in inputs {
        if input.sequence > *last_sequence {
            *last_sequence = input.sequence;

            if input.input.is_empty() {continue;}

            if input.input.contains(DirectionFlags::UP) {
                position[0] += 10 ;
            }
            if input.input.contains(DirectionFlags::DOWN) {
                position[0] -= 10 ;
            }
            if input.input.contains(DirectionFlags::LEFT) {
                position[1] -= 10 ;
            }
            if input.input.contains(DirectionFlags::RIGHT) {
                position[1] += 10 ;
            }
        }
    }
}