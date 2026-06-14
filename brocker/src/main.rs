use std::collections::HashMap;
use std::time::Duration;
use bytes::{Bytes, BytesMut};
use game_sockets::{GameConnection, GameNetworkEvent, GamePeer, GameStream};
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::{BroadcastPacket, ClientHandshakePacket, ClientInputBrokerPacket, PublishPacket};
use network_serialization::packets::Packet;
use network_serialization::packets::topic::{TopicTree, TopicTreeType};
use network_serialization::Serializable;

#[derive(Clone, Hash, Eq, PartialEq, Default)]
struct ConnectionData {
    connection: GameConnection,
    stream: GameStream,
}

struct BrokerState{
    game_peer : GamePeer,
    spatial_server: Option<ConnectionData>,
    client_to_connection: HashMap<u32, ConnectionData>,
    connection_to_client: HashMap<ConnectionData, u32>,
    game_connection_to_client: HashMap<GameConnection, u32>,
    client_to_subscribed_keys: HashMap<u32, Vec<Vec<u8>>>,
    shard_to_connection: HashMap<u32,ConnectionData>,
    next_client_id: u32,
}

impl BrokerState {
    fn new(game_peer: GamePeer) -> BrokerState {
        BrokerState{
            game_peer,
            spatial_server: None,
            client_to_connection: Default::default(),
            connection_to_client: Default::default(),
            game_connection_to_client: Default::default(),
            client_to_subscribed_keys: Default::default(),
            shard_to_connection: Default::default(),
            next_client_id: 0}
    }
}

#[tokio::main]
async fn main() {
    let backend = game_sockets::protocols::QuicBackend::new();
    let peer = game_sockets::GamePeer::new(backend);

    peer.listen("127.0.0.1",10001).unwrap();
    let mut broker = BrokerState::new(peer);

    loop {
        match broker.game_peer.poll() {
            Ok(Some(game_sockets::GameNetworkEvent::Message { connection, stream, data })) => {
                println!("Got message from peer: {:?}", connection);
                let msg = PacketMessage::read(data).unwrap();

                let connection_data = ConnectionData{ connection, stream };

                match msg.data {
                    PacketData::Subscribe(packet) => subscribe_client(&mut broker, connection_data, packet.client_id, packet.topic),
                    PacketData::Unsubscribe(packet) => unsubscribe_client(&mut broker, connection_data, packet.client_id, packet.topic),
                    PacketData::Publish(packet) => publish_shard_state(&mut broker, connection_data, packet),
                    PacketData::ClientInputBroker(packet) => handle_player_input(&mut broker, connection_data, packet),
                    PacketData::ClientHello(packet) => register_client(&mut broker, connection_data),

                    _ => println!("Unexpected message received")
                }
            }
            Ok(Some(game_sockets::GameNetworkEvent::Connected(conn))) => {
                println!("GameServer connected: {:?}", conn);
            }
            Ok(Some(game_sockets::GameNetworkEvent::Disconnected(conn))) => {
                cleanup_disconnected_client(&mut broker, conn);
            }
            Ok(Some(GameNetworkEvent::Error { connection, inner })) => {
                match inner {
                    game_sockets::GameSocketError::SendFailed { inner_msg } => {
                        println!("Send failed for {:?}: {}", connection, inner_msg);
                        cleanup_disconnected_client(&mut broker, connection);
                    }
                    e => println!("Other error: {:?}", e),
                }
            }
            Ok(Some(e)) => println!("Event: {:?}", e),
            Ok(None) => tokio::time::sleep(Duration::from_millis(10)).await,
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn register_connection(
    state: &mut BrokerState,
    connection_data: ConnectionData
) {
    let new_client_id = state.next_client_id;
    state.client_to_connection.insert(new_client_id, connection_data.clone());
    state.connection_to_client.insert(connection_data.clone(), new_client_id);
    state.game_connection_to_client.insert(connection_data.connection.clone(), new_client_id);
    state.next_client_id += 1;
    println!("J'ai register le client {}", new_client_id);
}

fn register_client(
    state: &mut BrokerState,
    connection_data: ConnectionData,
){
    println!("Register Client");

    let new_client_id = state.next_client_id.clone();
    register_connection(state, connection_data.clone());
    
    let packet = PacketMessage::new(PacketData::ClientHandshake(ClientHandshakePacket {}));
    state.game_peer.send(&connection_data.connection, &connection_data.stream, packet.write().unwrap()).unwrap();

    // todo : send data to shard

    // todo: remove, test code vvvvvvv
    /*if state.shard_to_connection.contains_key(&0u32) {
        let keys = state.client_to_subscribed_keys.entry(new_client_id).or_insert_with(Vec::new);
            keys.push(("entities/position/".to_owned()+ &*new_client_id.to_string()).to_string().into_bytes())
    } else {
        panic!("pas réussi à subscribe le client")
    }*/
    // todo: remove, test code ∧∧∧∧∧∧∧
}

fn cleanup_disconnected_client(
    state: &mut BrokerState,
    game_connection: GameConnection)
{
    let Some(client_id) = state.game_connection_to_client.get(&game_connection).copied() else {return;};
    let Some(connection_data) = state.client_to_connection.get(&client_id).clone() else {return;};

    state.connection_to_client.remove(&connection_data);
    state.game_connection_to_client.remove_entry(&game_connection);
    state.client_to_subscribed_keys.remove(&client_id);
    state.client_to_connection.remove(&client_id);
    if state.shard_to_connection.contains_key(&client_id) {
        state.shard_to_connection.remove(&client_id).unwrap();
    }

    println!("Client {} cleaned up", client_id);
}

fn register_spatial_server(
    state: &mut BrokerState,
    connection_data: ConnectionData,
){
    if state.spatial_server == None {
        println!("Register spatial server");
        state.spatial_server = Some(connection_data);
    }
}

fn register_shard(
    state: &mut BrokerState,
    connection_data: ConnectionData,
){
    match state.connection_to_client.get(&connection_data) {
        None => {
            println!("Register shard");
            // Dans ce cas, il le shard ne s'est jamais connecté au broker
            // On doit donc créé un nouveau client
            register_connection(state, connection_data.clone());
        }
        Some(shard_id) => {
            state.shard_to_connection.insert(*shard_id, connection_data.clone());
        }
    }
}

fn subscribe_client(
    state: &mut BrokerState,
    connection_data: ConnectionData,
    client_id: u32,
    topic: TopicTree,
){
    register_spatial_server(state, connection_data);

    let keys = topic.keys();
    let subscribed_key  = state.client_to_subscribed_keys.entry(client_id).or_insert(Vec::new());
    for key in keys {
        let key_str = String::from_utf8(key.clone()).unwrap();
        if key_str.ends_with("/*") {
            // Stocke sans le "/*" final → "entities/input"
            let trimmed = key_str.trim_end_matches("/*").to_string();
            subscribed_key.push(trimmed.into_bytes());
        } else {
            subscribed_key.push(key);
        }
        println!("Subscribed client {} to {:?}", client_id, subscribed_key.last());
    }
}

fn unsubscribe_client(
    state: &mut BrokerState,
    connection_data: ConnectionData,
    client_id: u32,
    topic: TopicTree
){
    register_spatial_server(state, connection_data);

    let keys = topic.keys();
    let subscribed_key = state.client_to_subscribed_keys.entry(client_id).or_insert(Vec::new());
    for key in keys {
        subscribed_key.retain(|x| *x != key);
    }
}

fn publish_shard_state(
    state: &mut BrokerState,
    connection_data: ConnectionData,
    packet: PublishPacket,
){
    println!("Publish Shard");
    register_shard(state, connection_data);

    for (client, keys) in &state.client_to_subscribed_keys {

        let mut topics_to_send = Vec::<TopicTree>::new();

        for tree_from_packet in packet.data.iter() {
            let mut potential_sub_tree = TopicTree::new_empty(tree_from_packet.name.clone());
            for key in keys {
                let Ok(key_string) = String::from_utf8(key.clone()) else {continue;};
                let Some(sub_tree) = tree_from_packet.get_sub_tree(&*key_string) else {continue};
                potential_sub_tree.merge(&sub_tree);
            }

            let TopicTreeType::Node(topic) = potential_sub_tree.item.clone() else {continue;};
            // S'il y a zéro donnée
            if topic.data.iter().count() == 0 {continue;}
            topics_to_send.push(potential_sub_tree);
        }

        if topics_to_send.len() == 0 {continue;}
        println!("on s'apprête à envoyer de la donnée à client {}",client);
        let Some(connection) = state.client_to_connection.get(client) else { continue; };
        println!("on s'apprête à envoyer de la donnée à connection {:?}",&connection.connection);
        let packet = PacketMessage::new(
            PacketData::Broadcast(
                BroadcastPacket{ data:topics_to_send },
            )
        );
        let bytes = packet.write().unwrap();

        println!("on envoie une donnée à quelqu'un");
        match state.game_peer.send(&connection.connection, &connection.stream, bytes){
            Ok(_) => {}
            Err(e) => {println!("Error during \"publish_shard state\": {}", e);}
        };
    }
}

fn handle_player_input(
    state: &mut BrokerState,
    connection_data: ConnectionData,
    packet: ClientInputBrokerPacket,
){
    let inputs = packet.inputs;

    let Some(client_id) = state.connection_to_client.get(&connection_data) else {return;};

    let mut tree_entities = TopicTree::new_empty("entities".to_string());
    let mut tree_input = TopicTree::new_empty("input".to_string());
    let mut bytes = BytesMut::new();;
    let _ = inputs.serialize(&mut bytes);
    tree_input.add_leaf(client_id.to_string(),bytes.to_vec());
    tree_entities.add_tree(tree_input);

    /* Code pour tester le hash et récupérer la donnée
    let hash = tree_player.clone().flatten();
    let key : String = client_id.to_string()+"/input";

    println!("tree: {:?}", hash.keys());
    let data = tree_player.get(&*key);
    println!("data: {:?}", data);

    println!("Player {:?} input: {:?}", client_id, input);*/

    let key_name : String = "entities/input/".to_owned() + &*client_id.to_string();
    let key_vec = Vec::<u8>::from(key_name.as_bytes());

    let packet = PacketMessage::new(
        PacketData::Broadcast(
            BroadcastPacket{
                data: vec!(tree_entities),
            }
        )
    );
    let bytes: Bytes = packet.write().unwrap();

    for (subscriber_id, subscribed_keys) in state.client_to_subscribed_keys.iter() {
        let matches = subscribed_keys.iter().any(|key| {
            let key_str = String::from_utf8(key.clone()).unwrap_or_default();
            key_name.starts_with(&key_str)
        });

        if matches {
            let Some(connection) = state.client_to_connection.get(subscriber_id) else { continue; };
            state.game_peer.send(
                &connection.connection,
                &connection.stream,
                bytes.clone()
            ).unwrap();
        }
    }
}