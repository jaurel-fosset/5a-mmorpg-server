use std::cmp::PartialEq;
use std::net;
use std::str::FromStr;
use std::sync;
use lazy_static::lazy_static;
use game_sockets as gs;
use network_serialization::Deserializable;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::{BroadcastPacket, SubscribePacket, UnsubscribePacket};
use network_serialization::packets::Packet;
use network_serialization::packets::spatial_server::*;
use network_serialization::packets::topic::{TopicTree, TopicTreeType};
use crate::network_object::entity::Entity;
use crate::network_object::shard::ShardId;


lazy_static!
{
    pub static ref SOCKET: sync::Mutex<NetworkGlobalState> = sync::Mutex::new(NetworkGlobalState::new());
}

pub enum NetworkEvent {
    ShardCreation(Vec<net::Ipv6Addr>),
    ShardDestruction(net::Ipv6Addr),
    PositionUpdate(Vec<(u32, f32, f32)>),
}

pub struct NetworkGlobalState
{
    orchestrator: OrchestratorConnection,
    redis_ip: Option<net::Ipv6Addr>,
    broker: Option<BrokerSocket>,
}

impl NetworkGlobalState
{
    pub fn new() -> Self
    {
        Self
        {
            orchestrator: OrchestratorConnection::new(),
            redis_ip: None,
            broker: None,
        }
    }

    fn broker_send(&self, bytes: bytes::Bytes) -> Result<(), NetworkError>
    {
        let broker = self.broker.as_ref().ok_or(NetworkError::ConnectionPartiallyInitialised)?;
        broker.send(bytes)
    }

    pub fn subscribe(&self, id: u32, topic: TopicTree) -> Result<(), NetworkError>
    {
        let packet = PacketMessage::new
        (
            PacketData::Subscribe
            (
                SubscribePacket
                {
                    client_id: id,
                    topic,
                }
            )
        ).write().unwrap();

        self.broker_send(packet)
    }

    pub fn unsubscribe(&self, id: u32, topic: TopicTree) -> Result<(), NetworkError>
    {
        let packet = PacketMessage::new
        (
            PacketData::Unsubscribe
            (
                UnsubscribePacket
                {
                    client_id: id,
                    topic,
                }
            )
        ).write().unwrap();

        self.broker_send(packet)
    }


    pub fn poll_once(&mut self) -> Option<NetworkEvent>
    {
        match self.orchestrator.poll_single()
        {
            None => return None,
            Some(packet) =>
            {
                match packet.data
                {
                    PacketData::OrchestratorHello(data) =>
                    {
                        self.redis_ip = Some(data.redis_dns);
                        self.broker = Some(BrokerSocket::new(data.broker, 3000)?);
                    }
                    PacketData::ShardCreation(data) => return Some(NetworkEvent::ShardCreation(data.shards)),
                    PacketData::ShardDestruction(data) => return Some(NetworkEvent::ShardDestruction(data.shard)),
                    _ => (),
                }
            }
        };
        

        let broker = match self.broker
        {
            Some(ref mut broker) => broker,
            None => return None,
        };

        let packet = match broker.poll_single()
        {
            Some(packet) => packet,
            None => return None,
        };

        match packet.data
        {
            PacketData::Broadcast(data) =>
            {
                Some(NetworkEvent::PositionUpdate(position_update_broadcast(data)?.collect()))
            }
            _ => None,
        }
    }

    pub fn request_more_shards(&self, amount: u64)
    {
        let packet = PacketMessage::new(
            PacketData::AllocateShards(AllocateShardsPacket::new(amount))
        );
        let bytes = packet.write().unwrap();

        match self.orchestrator.send(bytes)
        {
            Ok(_) => (),
            Err(_) => (),
        }
    }

    pub fn switch_authority(&self, new_shard: ShardId, old_shard: ShardId, entity: &mut Entity)
    {
        if let Some(broker) = &self.broker
        {
            let packet = PacketMessage::new(
                PacketData::AuthoritySwitch(
                    AuthoritySwitchPacket::new(old_shard.ip(), new_shard.ip(), entity.id().0)
                )
            );
            let bytes = packet.write().unwrap();

            match broker.send(bytes)
            {
                Ok(_) => (),
                Err(_) => (),
            }
        }
    }
}

const ORCHESTRATOR_PORT: u16 = 4000;

struct OrchestratorConnection
{
    socket: gs::GamePeer,
    connection: Option<gs::GameConnection>,
    command_stream: Option<gs::GameStream>,
}

impl OrchestratorConnection
{
    pub fn new() -> Self
    {
        let backend = gs::protocols::QuicBackend::new();
        let socket = gs::GamePeer::new(backend);
        socket.connect("0.0.0.0", ORCHESTRATOR_PORT)
            .unwrap();

        Self
        {
            socket,
            connection: None,
            command_stream: None,
        }
    }

    pub fn send(&self, bytes: bytes::Bytes) -> Result<(), NetworkError>
    {
        if let Some(connection) = &self.connection && let Some(command_stream) = &self.command_stream
        {
            self.socket.send(connection, command_stream, bytes).map_err(|_| { NetworkError::SendError })
        }
        else
        {
            Err(NetworkError::ConnectionPartiallyInitialised)
        }
    }
    
    pub fn poll_single(&mut self) -> Option<PacketMessage>
    {
        let event = self.socket.poll();

        let event = match event
        {
            Ok(event) => event,
            Err(error) =>
                {
                    println!("[Network] {}", error);
                    return None;
                }
        };

        let event = match event
        {
            Some(event) => event,
            None => return None,
        };

        match event
        {
            gs::GameNetworkEvent::Connected(connection) =>
            {
                if self.connection.is_some() { return None; }

                self.connection = Some(connection);
                let _ = self.socket.create_stream(connection, gs::GameStreamReliability::Reliable);
                None
            }
            gs::GameNetworkEvent::Disconnected(_) => None,
            gs::GameNetworkEvent::Message { connection, stream, data } =>
            {
                if self.connection != Some(connection) { return None; }
                if self.command_stream != Some(stream) { return None; }

                let msg = PacketMessage::read(data).unwrap();
                Some(msg)
            }
            gs::GameNetworkEvent::Error { .. } => None,
            gs::GameNetworkEvent::StreamCreated(connection, stream) =>
            {
                if self.connection != Some(connection) { return None; }
                if self.command_stream.is_some() { return None; }

                self.command_stream = Some(stream);
                None
            }
            gs::GameNetworkEvent::StreamClosed(_, _) => None,
        }
    }
}

struct BrokerSocket
{
    socket: gs::GamePeer,
    connection: Option<gs::GameConnection>,
    command_stream: Option<gs::GameStream>,
}

impl BrokerSocket
{
    pub fn new(address: net::Ipv6Addr, port: u16) -> Option<BrokerSocket>
    {
        let backend = gs::protocols::QuicBackend::new();
        let socket = gs::GamePeer::new(backend);
        socket.connect(address.to_string().as_str(), port).ok()?;

        Some(Self
        {
            socket,
            connection: None,
            command_stream: None,
        })
    }
    
    pub fn poll_single(&mut self) -> Option<PacketMessage>
    {
        let event = self.socket.poll();
        
        let event = match event
        {
            Ok(event) => event,
            Err(error) =>
            {
                println!("[Network] {}", error);
                return None;
            }
        };
        
        let event = match event
        {
            Some(event) => event,
            None => return None,
        };
        
        match event
        {
            gs::GameNetworkEvent::Connected(connection) =>
            {
                if self.connection.is_some() { return None; }
                
                self.connection = Some(connection);
                let _ = self.socket
                    .create_stream(connection, gs::GameStreamReliability::Reliable);

                None
            }
            gs::GameNetworkEvent::Disconnected(_) => None,
            gs::GameNetworkEvent::Message { connection, stream, data } => PacketMessage::read(data).ok(),
            gs::GameNetworkEvent::Error { .. } => None,
            gs::GameNetworkEvent::StreamCreated(connection, stream) =>
            {
                if self.connection != Some(connection) { return None; }
                if self.command_stream.is_some() { return None; }
                
                self.command_stream = Some(stream);
                None
            }
            gs::GameNetworkEvent::StreamClosed(_, _) => None,
        }
    }

    pub fn send(&self, bytes: bytes::Bytes) -> Result<(), NetworkError>
    {
        if let Some(connection) = &self.connection && let Some(command_stream) = &self.command_stream
        {
            self.socket
                .send(connection, command_stream, bytes)
                .map_err(|_| NetworkError::SendError)
        }
        else
        {
            Err(NetworkError::ConnectionPartiallyInitialised)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum NetworkError
{
    #[error("Error while sending the message")]
    SendError,
    #[error("Connection partially initialised")]
    ConnectionPartiallyInitialised,
}

fn parse_broadcast(packet: BroadcastPacket)
{
    if packet.topic.name == "orchestrator"
    {
        server_allocation_broadcast(packet);
    }
    else if packet.topic.name == "entities"
    {

    }
    else
    {
        eprintln!("[Network] Received unexpected broadcast packet {}", packet.topic.name);
    }
}

fn server_allocation_broadcast(packet: BroadcastPacket) -> (Option<impl Iterator<Item=u32>>, Option<impl Iterator<Item=u32>>)
{
    let creations = packet.topic
        .get_sub_tree("server_creations")
        .and_then(|creations|
            {
                match get_children(creations)
                {
                    Some(creations) => Some(broadcast_extract_shard(creations)),
                    None =>
                    {
                        eprintln!("[Network] orchestrator:server_creations is a leaf (expected children)");
                        None
                    }
                }
            });

    let deletions = packet.topic
        .get_sub_tree("server_deletions")
        .and_then(|deletions|
        {
            match get_children(deletions)
            {
                Some(deletions) => Some(broadcast_extract_shard(deletions)),
                None =>
                {
                    eprintln!("[Network] orchestrator:server_deletions is a leaf (expected children)");
                    None
                }
            }
        });

    (creations, deletions)
}

fn get_children(node: TopicTree) -> Option<impl Iterator<Item=(String, Vec<u8>)>>
{
    let children = match node.item
    {
        TopicTreeType::Leaf(_) => return None,
        TopicTreeType::Node(node) => node.data,
    };

    let children = children
        .into_iter()
        .flat_map(|topic|
            {
                match topic.item
                {
                    TopicTreeType::Node(_) => None,
                    TopicTreeType::Leaf(node) => Some((topic.name, node.data())),
                }
            });

    Some(children)
}

fn broadcast_extract_shard<T>(data: T) -> impl Iterator<Item=u32>
    where
        T: IntoIterator<Item=(String, Vec<u8>)>,
{
    data.into_iter()
        .flat_map(|(topic_name, node_data)|
            {
                let shard_id = match u32::from_str(&topic_name)
                {
                    Ok(client_id) => client_id,
                    Err(_) => return None,
                };

                let server_type: ServerType = ServerType::try_from(*node_data.get(0)?).ok()?;

                Some((shard_id, server_type))
            })
        .flat_map(|(shard_id, server_type)|
            {
                if server_type != ServerType::Shard { return None; }

                Some(shard_id)
            })
}


fn position_update_broadcast(packet: BroadcastPacket) -> Option<impl Iterator<Item=(u32, f32, f32)>>
{
    let positions = match packet.topic.get_sub_tree("positions")
    {
        Some(positions) => positions,
        None =>
        {
            eprintln!("[Network] Received ill-formed broadcast packet");
            return None;
        }
    };

    let positions = match get_children(positions)
    {
        Some(positions) => positions,
        None =>
        {
            eprintln!("[Network] entities:positions is a leaf (expected childrens)");
            return None;
        }
    };

    Some(broadcast_extract_positions(positions))
}

fn broadcast_extract_positions<T>(data: T) -> impl Iterator<Item=(u32, f32, f32)>
where
    T: IntoIterator<Item=(String, Vec<u8>)>,
{
    data
        .into_iter()
        .flat_map(|(name, data)|
        {
            let mut bytes = bytes::Bytes::from(data);
            let x = match f32::deserialize(&mut bytes)
            {
                Ok(x) => x,
                Err(_) => return None,
            };
            let y = match f32::deserialize(&mut bytes)
            {
                Ok(x) => x,
                Err(_) => return None,
            };

            let client_id = match u32::from_str(&name)
            {
                Ok(client_id) => client_id,
                Err(_) => return None,
            };

            Some((client_id, x, y))
        })
}

#[repr(u8)]
#[derive(int_enum::IntEnum, Debug, Eq, PartialEq)]
enum ServerType
{
    Orchestrator = 0,
    Broker,
    Spatial,
    Shard,
}