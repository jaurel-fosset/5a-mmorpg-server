use crate::inputs::Client;
use std::net::Ipv4Addr;
use bevy::app::App;
use bevy::prelude::*;
use bytes::Bytes;
use game_sockets::{GameConnection, GamePeer, GameSocketError, GameStream};
use network_serialization::Deserializable;
use network_serialization::input::InputData;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::{ClientHelloPacket, NetworkId};
use network_serialization::packets::Packet;
use network_serialization::packets::topic::TopicTreeType;
use crate::inputs::InputStore;
use crate::network::{orchestrator, NetworkUpdate};

pub struct BrokerPlugin;

impl Plugin for BrokerPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(NetworkUpdate, Self::receive_position);
    }
}

impl BrokerPlugin
{
    fn receive_position(mut commands: Commands, broker: Option<ResMut<BrokerPeer>>, mut input_store: ResMut<InputStore>)
    {
        let mut broker = match broker
        {
            Some(broker) => broker,
            None => return,
        };

        while let Some(event) = broker.game_peer.poll().transpose()
        {
            let event = match event
            {
                Ok(event) => event,
                Err(error) =>
                    {
                        error!("Broker : Error with a received packet : {}", error);
                        continue;
                    },
            };

            match event
            {
                game_sockets::GameNetworkEvent::Connected(connection) =>
                {
                }
                game_sockets::GameNetworkEvent::Disconnected(_) => {}
                game_sockets::GameNetworkEvent::Message { data, .. } =>
                {
                    let msg = PacketMessage::read(data).unwrap();
                    match msg.data
                    {
                        PacketData::Broadcast(packet) => for tree in packet.data
                        {
                            // On récupère les inputs
                            let Some(input_tree) = tree.get_child("input") else { break; };
                            let TopicTreeType::Node(input_topic_node) = &input_tree.item else { break; };

                            for topic_tree in &input_topic_node.data
                            {
                                let TopicTreeType::Leaf(leaf) = &topic_tree.item else { continue; };
                                println!("topic_tree.name: {:?}", topic_tree.name);

                                let Ok(client_id) = topic_tree.name.parse::<u32>() else { continue; };

                                let data = &leaf.data;
                                let mut bytes : Bytes = Bytes::copy_from_slice(data);
                                let Ok(inputs) = <[InputData;16]>::deserialize(&mut bytes) else {continue;};
                                
                                if !input_store.contains_client(client_id)
                                {
                                    commands.spawn((Transform::from_xyz(2f32,2f32,0f32), Client::new(client_id)));
                                }

                                input_store.add_input(client_id, inputs);
                            }
                        },
                        _ => {}
                    }
                }
                game_sockets::GameNetworkEvent::Error { .. } => {}
                game_sockets::GameNetworkEvent::StreamCreated(connection, stream) =>
                {
                }
                game_sockets::GameNetworkEvent::StreamClosed(_, _) => {}
            }
        }
    }
}

#[derive(Debug, Resource)]
pub struct BrokerPeer {
    game_peer: GamePeer,
    connection: GameConnection,
    stream: GameStream,
}

impl BrokerPeer
{
    pub(crate) fn send(&self, bytes: bytes::Bytes) -> Result<(), BrokerPeerError> {
        self.game_peer.send(&self.connection, &self.stream, bytes)
            .map_err(|_| BrokerPeerError::SendFail)?;

        Ok(())
    }
}

impl BrokerPeer
{
    pub fn new(broker_ip: Ipv4Addr) -> Self
    {
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

        Self
        {
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