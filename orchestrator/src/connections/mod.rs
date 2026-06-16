pub mod broker;
pub mod spatial;
pub mod shards;

use std::time::Duration;
use bollard::Docker;
use game_sockets as gs;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::orchestrator::OrchestratorHelloPacket;

async fn init_connection(ip: &str, port: u16) -> (gs::GamePeer, gs::GameConnection, gs::GameStream)
{
    let mut socket =
    {
        let backend = gs::protocols::QuicBackend::new();
        gs::GamePeer::new(backend)
    };

    socket.connect(ip, port).unwrap();

    let connection = loop
    {
        if let Ok(Some(game_sockets::GameNetworkEvent::Connected(conn))) = socket.poll()
        {
            break conn;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };

    socket.create_stream(connection.clone(), game_sockets::GameStreamReliability::Unreliable).unwrap();
    let stream: game_sockets::GameStream = loop
    {
        if let Ok(Some(game_sockets::GameNetworkEvent::StreamCreated(_, stream))) = socket.poll()
        {
            break stream;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };

    // TODO : Send hello packet
    // let hello_packet = PacketMessage::new(
    //     PacketData::OrchestratorHello(
    //         OrchestratorHelloPacket
    //         {
    //             orchestrator: (),
    //             redis_dns: (),
    //             broker: (),
    //         }
    //     )
    // );

    (socket, connection, stream)
}

async fn get_docker_ip(docker: &mut Docker, id: &str) -> String
{
    let inspect = docker.inspect_container(id, None).await.unwrap();

    inspect
        .network_settings.unwrap()
        .networks.unwrap()
        .get("mmorpg-server_default").unwrap()
        .ip_address.clone().unwrap()
}