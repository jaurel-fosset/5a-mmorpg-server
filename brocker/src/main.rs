use std::collections::HashMap;
use std::time::Duration;
use game_sockets::{GameConnection, GamePeer, GameStream};
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::{BroadcastPacket, ClientInputBrokerPacket};
use network_serialization::packets::Packet;
use network_serialization::packets::shard::ClientInputShardPacket;

#[derive(Clone, Hash, Eq, PartialEq, Default)]
struct ConnectionData {
    connection: GameConnection,
    stream: GameStream,
}

struct BrokerState{
    game_peer : GamePeer,
    subscription: HashMap<[u8;32],Vec<u32>>,
    spatial_server: Option<ConnectionData>,
    shard_to_connection: HashMap<[u8;32],ConnectionData>,
    client_to_connection: HashMap<u32, ConnectionData>,
    connection_to_client: HashMap<ConnectionData, u32>,
    client_to_shard: HashMap<u32, [u8;32]>,
    next_client_id: u32,
}

impl BrokerState {
    fn new(game_peer: GamePeer) -> BrokerState {
        BrokerState{
            game_peer,
            subscription: Default::default(),
            spatial_server: None,
            shard_to_connection: Default::default(),
            client_to_connection: Default::default(),
            connection_to_client: Default::default(),
            client_to_shard: Default::default(),
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
                    PacketData::Publish(packet) => publish_shard_state(&mut broker, connection_data, packet.topic, packet.payload),
                    PacketData::ClientInputBroker(packet) => handle_player_input(&mut broker, connection_data, packet.input),
                    PacketData::RegisterPlayer(packet) => register_player(&mut broker, connection_data),
                    _ => println!("Unexpected message received")
                }
            }
            Ok(Some(game_sockets::GameNetworkEvent::Connected(conn))) => {
                println!("GameServer connected: {:?}", conn);
            }
            Ok(Some(e)) => println!("Event: {:?}", e),
            Ok(None) => tokio::time::sleep(Duration::from_millis(10)).await,
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn register_player(
    state: &mut BrokerState,
    connection_data: ConnectionData,
){
    let client_id = state.next_client_id;
    state.client_to_connection.insert(client_id, connection_data.clone());
    state.connection_to_client.insert(connection_data, client_id);

    // todo : send data to shard
    state.next_client_id += 1;
}

fn register_spatial_server(
    state: &mut BrokerState,
    connection_data: ConnectionData,
){
    if state.spatial_server == None {
        state.spatial_server = Some(connection_data);
    }
}

fn register_shard(
    state: &mut BrokerState,
    connection_data: ConnectionData,
    topic: [u8; 32],
){
    if let None = state.shard_to_connection.get(&topic) {
        state.shard_to_connection.insert(topic, connection_data);
    }
}

fn subscribe_client(
    state: &mut BrokerState,
    connection_data: ConnectionData,
    client_id: u32,
    topic: [u8;32]
){
    register_spatial_server(state, connection_data);

    let clients = state.subscription.entry(topic).or_insert_with(Vec::new);
    clients.push(client_id);

    state.client_to_shard.insert(client_id, topic);
}

fn unsubscribe_client(
    state: &mut BrokerState,
    connection_data: ConnectionData,
    client_id: u32,
    topic: [u8;32]
){
    register_spatial_server(state, connection_data);

    let clients = state.subscription.entry(topic).or_insert_with(Vec::new);
    clients.retain(|&x| x != client_id);

    if state.client_to_shard.get(&client_id) == Some(&topic) {
        state.client_to_shard.remove(&client_id);
    }
}

fn publish_shard_state(
    state: &mut BrokerState,
    connection_data: ConnectionData,
    topic: [u8;32],
    payload: Vec<u8>,
){
    register_shard(state, connection_data, topic);

    let clients = state.subscription.entry(topic).or_insert_with(Vec::new);
    let packet = PacketMessage::new(
        PacketData::Broadcast(
            BroadcastPacket{payload}
        )
    );
    let bytes = packet.write().unwrap();

    for client_id in clients {
        let player_data = state.client_to_connection.get(&client_id);
        match player_data {
            Some(data) => state.game_peer.send(&data.connection,&data.stream,bytes.clone()).unwrap(),
            None => (),
        }
    }
}

fn handle_player_input(
    state: &mut BrokerState,
    connection_data: ConnectionData,
    input: [u8; 16]
){
    let Some(client_id) = state.connection_to_client.get(&connection_data) else {return;};
    let Some(shard_topic) = state.client_to_shard.get(&client_id) else {return;};
    let Some(shard_connection) = state.shard_to_connection.get(shard_topic) else {return;};

    let packet = PacketMessage::new(
        PacketData::ClientInputShard(
            ClientInputShardPacket { client_id: *client_id, input, }
        )
    );
    let bytes = packet.write().unwrap();

    state.game_peer.send(&shard_connection.connection, &shard_connection.stream, bytes).unwrap();
}