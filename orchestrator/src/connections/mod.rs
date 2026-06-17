pub mod broker;
pub mod spatial;
pub mod shards;
pub mod redis;

use std::net::Ipv4Addr;
use std::str::FromStr;
use std::time::Duration;
use bollard::Docker;
use game_sockets as gs;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::orchestrator::OrchestratorHelloPacket;
use network_serialization::packets::Packet;

async fn init_connection(ip: Ipv4Addr, port: u16, orchestrator_ip: Ipv4Addr, redis_dns_ip: Ipv4Addr, broker_ip: Ipv4Addr) -> (gs::GamePeer, gs::GameConnection, gs::GameStream)
{
    let mut socket =
    {
        let backend = gs::protocols::QuicBackend::new();
        gs::GamePeer::new(backend)
    };

    socket.connect(&ip.to_string(), port).unwrap();

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

    let hello_packet = PacketMessage::new
    (
        PacketData::OrchestratorHello
        (
            OrchestratorHelloPacket
            {
                orchestrator: orchestrator_ip,
                redis_dns: redis_dns_ip,
                broker: broker_ip,
            }
        )
    ).write().unwrap();
    socket.send(&connection, &stream, hello_packet).unwrap();

    (socket, connection, stream)
}

async fn get_docker_ip(docker: &mut Docker, id: &str) -> Ipv4Addr
{
    tokio::time::sleep(Duration::from_secs(2)).await;

    let inspect = docker.inspect_container(id, None).await.unwrap();

    let ip_string = inspect
        .network_settings.unwrap()
        .networks.unwrap()
        .get("mmorpg-server_default").unwrap()
        .ip_address.clone().unwrap();

    Ipv4Addr::from_str(&ip_string).unwrap()
}