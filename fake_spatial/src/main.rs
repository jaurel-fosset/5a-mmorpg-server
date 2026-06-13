use std::time::Duration;
use game_sockets::{GameConnection, GameNetworkEvent, GameStream, GameStreamReliability};
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::{ClientHelloPacket, SubscribePacket};
use network_serialization::packets::Packet;
use network_serialization::packets::topic::TopicTree;

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

    let packet = PacketMessage::new(
        PacketData::ClientHello(ClientHelloPacket {}),
    );
    let bytes = packet.write().unwrap();
    peer.send(&conn,&stream,bytes).unwrap();
    std::thread::sleep(Duration::from_millis(100));




    // on va subscribe le client 1 (fake_shard) aux input du client 2 (client)
    let mut tree_entities = TopicTree::new_empty("entities".to_string());
    let mut tree_input = TopicTree::new_empty("input".to_string());
    tree_input.add_leaf("2".to_string(),Vec::new());
    tree_entities.add_tree(tree_input);

    let packet = PacketMessage::new(
        PacketData::Subscribe(
            SubscribePacket{
                client_id: 1,
                topic: tree_entities,
            }
        )
    );
    let bytes = packet.write().unwrap();
    peer.send(&conn,&stream,bytes).unwrap();
    std::thread::sleep(Duration::from_millis(100));

    // on va subscribe le client 2 (client) à sa position géré par le client 1 (fake_shard)
    let mut tree_entities = TopicTree::new_empty("entities".to_string());
    let mut tree_position = TopicTree::new_empty("position".to_string());
    tree_position.add_leaf("2".to_string(),Vec::new());
    tree_entities.add_tree(tree_position);

    let packet = PacketMessage::new(
        PacketData::Subscribe(
            SubscribePacket{
                client_id: 2,
                topic: tree_entities,
            }
        )
    );
    let bytes = packet.write().unwrap();
    peer.send(&conn,&stream,bytes).unwrap();
    std::thread::sleep(Duration::from_millis(100));
}
