use std::net::Ipv4Addr;
use bevy::app::App;
use bevy::prelude::*;
use game_sockets::{GameConnection, GamePeer, GameSocketError, GameStream};
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::{ClientHelloPacket, NetworkId};
use network_serialization::packets::Packet;

struct BrokerPlugin;

impl Plugin for BrokerPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Debug, Resource)]
pub struct BrokerPeer {
    game_peer: GamePeer,
    connection: GameConnection,
    stream: GameStream,
}

impl BrokerPeer
{
    fn send(&self, bytes: bytes::Bytes) -> Result<(), BrokerPeerError> {
        self.game_peer.send(&self.connection, &self.stream, bytes)
            .map_err(|_| BrokerPeerError::SendFail)?;

        Ok(())
    }
}

impl BrokerPeer {
    pub fn new(broker_ip: Ipv4Addr) -> Self {
        let backend = game_sockets::protocols::QuicBackend::new();
        let mut peer = game_sockets::GamePeer::new(backend);
        peer.connect(&broker_ip.to_string(), 10_001)
            .unwrap();

        let connection = loop
        {
            if let Ok(Some(game_sockets::GameNetworkEvent::Connected(conn))) = peer.poll()
            {
                break conn;
            }
        };

        peer.create_stream(connection.clone(), game_sockets::GameStreamReliability::Unreliable).unwrap();
        let stream: game_sockets::GameStream = loop
        {
            if let Ok(Some(game_sockets::GameNetworkEvent::StreamCreated(_, stream))) = peer.poll()
            {
                break stream;
            }
        };

        Self {
            game_peer: peer,
            connection,
            stream,
        }
    }

    pub fn send_client_hello(&mut self)
    {
        let packet = PacketMessage::new
        (
            PacketData::ClientHello
            (
                ClientHelloPacket
                {
                    client_type: NetworkId::Shard,
                }
            )
        ).write().unwrap();

        println!("Sent client hello {:?}", packet);
        _ = self.send(packet);
    }
}

pub enum BrokerPeerError
{
    SendFail,
}