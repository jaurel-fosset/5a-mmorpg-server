use bytes::{Buf, BufMut, Bytes, BytesMut};
use network_serialization::packets::Packet;
use network_serialization::packets::spatial_server;
use network_serialization::packets::orchestrator;
use network_serialization::{Deserializable, Serializable, SerializationError};
use crate::network_connection::packet::OrchestratorPacket::{Hello, ShardCreation, ShardDestruction};

pub enum OrchestratorPacket
{
    Hello(orchestrator::HelloPacket),
    ShardCreation(spatial_server::ShardCreationPacket),
    ShardDestruction(spatial_server::ShardDestructionPacket),
}

impl Packet for OrchestratorPacket
{
    fn read(mut bytes: Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let packet_type = OrchestratorPacketType::deserialize(&mut bytes)?;
        match packet_type
        {
            OrchestratorPacketType::Hello => 
            {
                Ok(Hello(orchestrator::HelloPacket::deserialize(&mut bytes)?))
            }
            OrchestratorPacketType::ShardCreation =>
            {
                Ok(ShardCreation(spatial_server::ShardCreationPacket::deserialize(&mut bytes)?))
            }
            OrchestratorPacketType::ShardDestruction =>
            {
                Ok(ShardDestruction(spatial_server::ShardDestructionPacket::deserialize(&mut bytes)?))
            }
        }
    }

    fn write(self) -> Result<Bytes, SerializationError>
    {
        let mut buffer = bytes::BytesMut::new();
        
        match self
        { 
            Hello(packet) => packet.serialize(&mut buffer)?,
            ShardCreation(packet) => packet.serialize(&mut buffer)?,
            ShardDestruction(packet) => packet.serialize(&mut buffer)?,
        };
        
        Ok(buffer.freeze())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, int_enum::IntEnum)]
enum OrchestratorPacketType
{
    Hello = 0,
    ShardCreation,
    ShardDestruction,
}


impl Serializable for OrchestratorPacketType
{
    fn serialize(self, stream: &mut BytesMut) -> Result<(), SerializationError>
    {
        stream.put_u8(u8::from(self));
        Ok(())
    }
}

impl Deserializable for OrchestratorPacketType
{
    fn deserialize(stream: &mut Bytes) -> Result<Self, SerializationError>
    {
        stream.get_u8().try_into().map_err(|_| { SerializationError::InvalidDeserializationState })
    }
}

struct InvalidPacketType;