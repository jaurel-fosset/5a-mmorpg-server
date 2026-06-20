use std::f32;
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
use network_serialization::packets::topic::{TopicTree, TopicTreeType};
use crate::client;
use crate::client::NotAuthoritative;
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
    fn receive_position(
        mut commands: Commands,
        broker: Option<ResMut<BrokerPeer>>,
        mut input_store: ResMut<InputStore>,
        mut clients: Query<(Entity, &Client, &mut Transform)>
    )
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
                            //print!("r");
                            if let Some(position_tree) = tree.get_child("position") {
                                //print!("p");
                                let TopicTreeType::Node(position_topic_node) = &position_tree.item else { break; };

                                for topic_tree in &position_topic_node.data
                                {
                                    let TopicTreeType::Leaf(leaf) = &topic_tree.item else { continue; };

                                    let Ok(client_id) = topic_tree.name.parse::<u32>() else { continue; };

                                    let data = &leaf.data;
                                    let mut bytes : Bytes = Bytes::copy_from_slice(data);
                                    let Ok(x) = f32::deserialize(&mut bytes) else {continue;};
                                    let Ok(y) = f32::deserialize(&mut bytes) else {continue;};

                                    if let Some((_, _, mut transform)) = clients.iter_mut().find(|(_, client, _)| client.id() == client_id) {
                                        // L'entité existe déjà (spawnée par les inputs) → met à jour la position
                                        transform.translation.x = x;
                                        transform.translation.y = y;
                                    } else {
                                        // L'entité n'existe pas → spawn à la bonne position
                                        commands.spawn((Transform::from_xyz(x, y, 0f32), Client::new(client_id), client::NotAuthoritative));
                                    }
                                }
                            }

                            // On récupère les inputs
                            if let Some(input_tree) = tree.get_child("input") {
                                //print!("i");
                                let TopicTreeType::Node(input_topic_node) = &input_tree.item else { break; };

                                for topic_tree in &input_topic_node.data
                                {
                                    let TopicTreeType::Leaf(leaf) = &topic_tree.item else { continue; };
                                    //println!("topic_tree.name: {:?}", topic_tree.name);

                                    let Ok(client_id) = topic_tree.name.parse::<u32>() else { continue; };

                                    let data = &leaf.data;
                                    let mut bytes : Bytes = Bytes::copy_from_slice(data);
                                    let Ok(inputs) = <[InputData;16]>::deserialize(&mut bytes) else {continue;};

                                    if !input_store.contains_client(client_id)
                                        && !clients.iter().any(|(_, c, _)| c.id() == client_id)
                                    {
                                        commands.spawn((Transform::from_xyz(0f32,0f32,0f32), Client::new(client_id)));
                                    }

                                    input_store.add_input(client_id, inputs);
                                }
                            }

                            if let Some(authority_tree) = tree.get_child("authority")
                            {
                                let TopicTreeType::Node(authority_topic_node) = &authority_tree.item else { continue; };

                                let authority_gain_tree = authority_topic_node.data
                                    .iter()
                                    .find(|topic_tree_type| topic_tree_type.name == "gain" );

                                if let Some(authority_gain_tree) = authority_gain_tree
                                {
                                    handle_authority_gain(&mut commands, &mut clients, authority_gain_tree);
                                }


                                let authority_loss_tree = authority_topic_node.data
                                    .iter()
                                    .find(|topic_tree_type| topic_tree_type.name == "loss" );

                                if let Some(authority_loss_tree) = authority_loss_tree
                                {
                                    handle_authority_loss(&mut commands, &clients, authority_loss_tree);
                                }
                            }

                            println!("end");
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

fn handle_authority_gain(commands: &mut Commands, clients: &mut Query<(Entity, &Client, &mut Transform)>, authority_gain_tree: &TopicTree)
{
    let entity_nodes = match &authority_gain_tree.item
    {
        TopicTreeType::Node(entity_nodes) => entity_nodes,
        TopicTreeType::Leaf(_) => return,
    };

    for entity_node in &entity_nodes.data
    {
        let entity = match &entity_node.item
        {
            TopicTreeType::Node(_) => continue,
            TopicTreeType::Leaf(entity) => entity,
        };

        let entity_id = match entity_node.name.parse::<u32>()
        {
            Ok(entity_id) => entity_id,
            Err(_) => continue,
        };
        let entity_data = entity.data.clone();

        let mut bytes = Bytes::from(entity_data);
        let position_x = f32::deserialize(&mut bytes).unwrap();
        let position_y = f32::deserialize(&mut bytes).unwrap();

        let client = clients
            .iter_mut()
            .find(|(_, client, _)| client.id() == entity_id);
        match client
        {
            Some((entity, _, mut transform)) =>
            {
                transform.translation.x = position_x;
                transform.translation.y = position_y;
                commands.entity(entity).remove::<NotAuthoritative>();
            }
            None =>
            {
                commands.spawn(
                    (
                        Transform::from_xyz(position_x, position_y, 0f32),
                        Client::new(entity_id),
                    ));
            }
        }
    }
}

fn handle_authority_loss(commands: &mut Commands, clients: &Query<(Entity, &Client, &mut Transform)>, authority_gain_tree: &TopicTree)
{
    let entity_nodes = match &authority_gain_tree.item
    {
        TopicTreeType::Node(entity_nodes) => entity_nodes,
        TopicTreeType::Leaf(_) => return,
    };

    for entity_node in &entity_nodes.data
    {
        let entity = match &entity_node.item
        {
            TopicTreeType::Node(_) => continue,
            TopicTreeType::Leaf(entity) => entity,
        };

        let entity_id = match entity_node.name.parse::<u32>()
        {
            Ok(entity_id) => entity_id,
            Err(_) => continue,
        };

        let client = clients
            .iter()
            .find(|(_, client, _)| client.id() == entity_id);
        
        if let Some((entity, client, _)) = client
        {
            commands.entity(entity).insert(NotAuthoritative);
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