use std::cmp::PartialEq;
use std::net;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::time::Duration;
use game_sockets as gs;
use network_serialization::Deserializable;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::{BroadcastPacket, SubscribePacket, UnsubscribePacket};
use network_serialization::packets::game_server::HeartbeatPacket;
use network_serialization::packets::Packet;
use network_serialization::packets::spatial_server::*;
use network_serialization::packets::topic::{TopicTree, TopicTreeType};
use crate::network_object::entity::Entity;
use crate::network_object::shard::ShardId;


pub enum NetworkEvent
{
    ShardsUpdate(Vec<u32>, Vec<u32>),
    PositionUpdate(Vec<(u32, f32, f32)>),
}

pub struct NetworkGlobalState
{
    orchestrator: OrchestratorConnection,
    redis_ip: Option<net::Ipv4Addr>,
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

    pub fn send_heartbeat(&self) -> Result<(), NetworkError>
    {
        let packet = PacketMessage::new
        (
            PacketData::Heartbeat
            (
                HeartbeatPacket
                {
                    ip: net::Ipv4Addr::new(127, 0, 0, 1),
                    port: ORCHESTRATOR_PORT,
                    player_number: 0,
                    player_capacity: 0,
                    cpu_load: 0,
                    ram_load: 0,
                }
            )
        ).write().unwrap();

        self.orchestrator.send(packet)
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
            None => (),
            Some(packet) =>
            {
                match packet.data
                {
                    PacketData::OrchestratorHello(data) =>
                    {
                        self.redis_ip = Some(data.redis_dns);
                        self.broker = Some(BrokerSocket::new(data.broker)?);
                    }
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
                Some(self.parse_broadcast(data)?)
            }
            _ => None,
        }
    }

    pub fn request_more_shards(&self, amount: u64)
    {
        println!("[Spatial] Requesting moreShards: {}", amount);
        let packet = PacketMessage::new(
            PacketData::AllocateShards(AllocateShardsPacket::new(amount))
        );
        let bytes = packet.write().unwrap();

        match self.orchestrator.send(bytes)
        {
            Ok(_) => (),
            Err(e) => println!("[Spatial] Error sending to orchestrator: {:?}", e),
        }
    }

    pub fn switch_authority(&self, new_shard: ShardId, old_shard: ShardId, entity: &mut Entity)
    {
        if let Some(broker) = &self.broker
        {
            let packet = PacketMessage::new(
                PacketData::AuthoritySwitch(
                    AuthoritySwitchPacket::new(old_shard.id(), new_shard.id(), entity.id().0)
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

    fn parse_broadcast(&self, packet: BroadcastPacket) -> Option<NetworkEvent>
    {
        for tree in packet.data {
            if tree.name == "orchestrator"
            {
                let (created_shards, destroyed_shards) = server_allocation_broadcast(tree);
                let created_shards = created_shards.map(|shards| { shards.collect::<Vec<_>>() });
                let destroyed_shards = destroyed_shards.map(|shards| { shards.collect::<Vec<_>>() });

                if created_shards.is_none() && destroyed_shards.is_none() { return None; }

                let created_shards = created_shards.unwrap_or(Vec::new());
                let destroyed_shards = destroyed_shards.unwrap_or(Vec::new());
                return Some(NetworkEvent::ShardsUpdate(created_shards, destroyed_shards))
            }
            else if tree.name == "entities"
            {
                let positions = position_update_broadcast(tree)
                    .map(|positions| positions.collect())
                    .unwrap_or(Vec::new());

                return Some(NetworkEvent::PositionUpdate(positions))
            }
            else
            {
                eprintln!("[Network] Received unexpected broadcast packet {}", tree.name);
            }
        }
        None
    }

    fn broker_send(&self, bytes: bytes::Bytes) -> Result<(), NetworkError>
    {
        let broker = self.broker.as_ref().ok_or(NetworkError::ConnectionPartiallyInitialised)?;
        broker.send(bytes)
    }

    pub fn is_orchestrator_connected(&self) -> bool {
        self.orchestrator.is_connected()
    }
}

const ORCHESTRATOR_PORT: u16 = 50_000;

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
        socket.listen("0.0.0.0", ORCHESTRATOR_PORT)
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
                println!("[Network] Orchestrator Connected!");
                if self.connection.is_some() { return None; }

                self.connection = Some(connection);
                let _ = self.socket.create_stream(connection, gs::GameStreamReliability::Reliable);
                None
            }
            gs::GameNetworkEvent::Disconnected(_) => None,
            gs::GameNetworkEvent::Message { connection, stream, data } =>
            {
                //if self.connection != Some(connection) { return None; }
                //if self.command_stream != Some(stream) { return None; }

                let msg = PacketMessage::read(data).unwrap();
                println!("Orchestrator packet : {:?}", msg);
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

    pub fn is_connected(&self) -> bool {
        self.connection.is_some() && self.command_stream.is_some()
    }
}

const BROKER_PORT: u16 = 10_001;

struct BrokerSocket
{
    socket: gs::GamePeer,
    connection: Option<gs::GameConnection>,
    command_stream: Option<gs::GameStream>,
}

impl BrokerSocket
{
    pub fn new(address: net::Ipv4Addr) -> Option<BrokerSocket>
    {
        let backend = gs::protocols::QuicBackend::new();
        let mut socket = gs::GamePeer::new(backend);
        println!("[Network] {}:{}", address,BROKER_PORT);
        socket.connect(address.to_string().as_str(), BROKER_PORT).ok()?;

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
                println!("[Network] Broker Connected!");
                if self.connection.is_some() { return None; }
                
                self.connection = Some(connection);
                let _ = self.socket
                    .create_stream(connection, gs::GameStreamReliability::Reliable);

                None
            }
            gs::GameNetworkEvent::Disconnected(_) => None,
            gs::GameNetworkEvent::Message { connection: _, stream: _, data } => PacketMessage::read(data).ok(),
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

fn server_allocation_broadcast(tree: TopicTree) -> (Option<impl Iterator<Item=u32>>, Option<impl Iterator<Item=u32>>)
{
    let creations = tree
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

    let deletions = tree
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


fn position_update_broadcast(tree: TopicTree) -> Option<impl Iterator<Item=(u32, f32, f32)>>
{
    let positions = match tree.get_sub_tree("positions")
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