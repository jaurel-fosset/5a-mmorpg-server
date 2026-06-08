pub mod packet;

use std::net;
use std::sync;
use lazy_static::lazy_static;
use game_sockets as gs;
use network_serialization::packets::Packet;
use network_serialization::packets::spatial_server::*;
use crate::network_connection::packet::OrchestratorPacket;
use crate::network_object::entity::Entity;
use crate::network_object::shard::ShardId;


lazy_static!
{
    pub static ref SOCKET: sync::Mutex<NetworkGlobalState> = sync::Mutex::new(NetworkGlobalState::new());
}

pub struct NetworkGlobalState
{
    socket: gs::GamePeer,
    orchestrator: Option<OrchestratorConnection>,
    redis_ip: Option<net::Ipv6Addr>,
    broker: Option<BrokerSocket>,
}

impl NetworkGlobalState
{
    pub fn new() -> Self
    {
        let backend = gs::protocols::QuicBackend::new();
        let socket = gs::GamePeer::new(backend);

        Self
        {
            socket,
            orchestrator: None,
            redis_ip: None,
            broker: None,
        }
    }
    
    pub fn poll_once(&mut self)
    {
        let orchestrator = match &mut self.orchestrator
        {
            Some(orchestrator) => orchestrator,
            None => return
        };
        
        let packet = match orchestrator.poll_single()
        {
            None => return,
            Some(packet) => packet,
        };
        
        match packet
        {
            OrchestratorPacket::Hello(hello_packet) => 
            {
                self.redis_ip = Some(hello_packet.redis_dns);
            }
            OrchestratorPacket::ShardCreation(_) => 
            {
                // TODO : handle shard creation
            }
            OrchestratorPacket::ShardDestruction(_) => 
            {
                // TODO : handle shard deletion
            }
        }
    }

    pub fn request_more_shards(&self, amount: u64)
    {
        if let Some(orchestrator) = &self.orchestrator
        {
            let packet = AllocateShardsPacket::new(amount).write().unwrap();
            match orchestrator.send(packet)
            {
                Ok(_) => (),
                Err(_) => (),
            }
        }
    }

    pub fn switch_authority(&self, new_shard: ShardId, old_shard: ShardId, entity: &mut Entity)
    {
        if let Some(broker) = &self.broker
        {
            let authority_gain = AuthoritySwitchPacket::new(old_shard.ip(), new_shard.ip(), entity.id().0).write().unwrap();
            match broker.send(authority_gain)
            {
                Ok(_) => (),
                Err(_) => (),
            }
        }
    }
}

struct OrchestratorConnection
{
    socket: gs::GamePeer,
    connection: Option<gs::GameConnection>,
    command_stream: Option<gs::GameStream>,
}

impl OrchestratorConnection
{
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
    
    pub fn poll_single(&mut self) -> Option<OrchestratorPacket>
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
                
                let packet = match OrchestratorPacket::read(data)
                { 
                    Ok(packet) => packet,
                    Err(error) => return None,
                };
                
                Some(packet)
            }
            gs::GameNetworkEvent::Error { .. } => None,
            gs::GameNetworkEvent::StreamCreated(connection, stream) =>
            {
                if self.connection != Some(connection) { return None; }
                if self.command_stream.is_some() { return None; }

                self.command_stream = Some(stream);
                None
            }
            gs::GameNetworkEvent::StreamClosed(_, _) => None
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
    
    pub fn poll_single(&mut self)
    {
        let event = self.socket.poll();
        
        let event = match event
        {
            Ok(event) => event,
            Err(error) =>
            {
                println!("[Network] {}", error);
                return;
            }
        };
        
        let event = match event
        {
            Some(event) => event,
            None => return,
        };
        
        match event
        {
            gs::GameNetworkEvent::Connected(connection) =>
            {
                if self.connection.is_some() { return; }
                
                self.connection = Some(connection);
                let _ = self.socket.create_stream(connection, gs::GameStreamReliability::Reliable);
            }
            gs::GameNetworkEvent::Disconnected(_) => {}
            gs::GameNetworkEvent::Message { .. } =>
            {
                // TODO : read positions publish
            }
            gs::GameNetworkEvent::Error { .. } => {}
            gs::GameNetworkEvent::StreamCreated(connection, stream) =>
            {
                if self.connection != Some(connection) { return; }
                if self.command_stream.is_some() { return; }
                
                self.command_stream = Some(stream);
            }
            gs::GameNetworkEvent::StreamClosed(_, _) => {}
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
}

enum NetworkError
{
    SendError,
    ConnectionPartiallyInitialised,
}