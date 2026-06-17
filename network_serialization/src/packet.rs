use crate::packets::Packet;
use crate::{Deserializable, Serializable, SerializationError};
use crate::packets::broker::{BroadcastPacket, ClientInputBrokerPacket, PublishPacket, ClientHelloPacket, SubscribePacket, UnsubscribePacket, ClientHandshakePacket};
use crate::packets::game_server::HeartbeatPacket;
use crate::packets::orchestrator::OrchestratorHelloPacket;
use crate::packets::shard::ClientInputShardPacket;
use crate::packets::spatial_server::{AllocateShardsPacket, AuthoritySwitchPacket, DeAllocateShardsPacket, ShardCreationPacket, ShardDestructionPacket};

#[derive(Debug)]
pub struct PacketMessage {
    pub tag: u8,
    pub data: PacketData,
}

impl PacketMessage {
    pub fn new(data: PacketData) -> Self {
        let tag: u8 = data.tag();
        Self { tag, data }
    }
}

#[derive(Debug)]
pub enum PacketData {
    Subscribe(SubscribePacket),
    Unsubscribe(UnsubscribePacket),
    Publish(PublishPacket),
    Broadcast(BroadcastPacket),
    ClientInputBroker(ClientInputBrokerPacket),
    ClientHello(ClientHelloPacket),
    ClientHandshake(ClientHandshakePacket),
    ClientInputShard(ClientInputShardPacket),

    //=== SHARD ===
    AllocateShards(AllocateShardsPacket),
    DeAllocateShards(DeAllocateShardsPacket),
    ShardCreation(ShardCreationPacket),
    ShardDestruction(ShardDestructionPacket),
    AuthoritySwitch(AuthoritySwitchPacket),

    OrchestratorHello(OrchestratorHelloPacket),
    Heartbeat(HeartbeatPacket),
}

#[repr(u8)]
#[derive(Debug, int_enum::IntEnum)]
pub enum PacketTag {
    Subscribe = 0x00,
    Unsubscribe = 0x01,
    Publish = 0x02,
    Broadcast = 0x03,
    ClientInputBroker = 0x04,
    ClientHello = 0x05,
    ClientHandshake = 0x06,
    ClientInputShard = 0x07,

    AllocateShards = 0x10,
    DeAllocateShards = 0x11,
    ShardCreation = 0x12,
    ShardDestruction = 0x13,
    AuthoritySwitch = 0x14,

    OrchestratorHello = 0x20,
    Heartbeat = 0x21,
}


impl PacketData {
    pub fn tag(&self) -> u8 {
        match self {
            PacketData::Subscribe(_) => PacketTag::Subscribe as u8,
            PacketData::Unsubscribe(_) => PacketTag::Unsubscribe as u8,
            PacketData::Publish(_) => PacketTag::Publish as u8,
            PacketData::Broadcast(_) => PacketTag::Broadcast as u8,
            PacketData::ClientInputBroker(_) => PacketTag::ClientInputBroker as u8,
            PacketData::ClientHello(_) => PacketTag::ClientHello as u8,
            PacketData::ClientInputShard(_) => PacketTag::ClientInputShard as u8,
            PacketData::ClientHandshake(_) => PacketTag::ClientHandshake as u8,

            PacketData::AllocateShards(_) => PacketTag::AllocateShards as u8,
            PacketData::DeAllocateShards(_) => PacketTag::DeAllocateShards as u8,
            PacketData::ShardCreation(_) => PacketTag::ShardCreation as u8,
            PacketData::ShardDestruction(_) => PacketTag::ShardDestruction as u8,
            PacketData::AuthoritySwitch(_) => PacketTag::AuthoritySwitch as u8,

            PacketData::OrchestratorHello(_) => PacketTag::OrchestratorHello as u8,
            PacketData::Heartbeat(_) => PacketTag::Heartbeat as u8,
        }
    }
}

impl Packet for PacketMessage {
    fn read(mut bytes: bytes::Bytes) -> Result<Self, SerializationError> {
        let tag = u8::deserialize(&mut bytes)?;
        let packet_tag = PacketTag::try_from(tag).map_err(|_| SerializationError::InvalidDeserializationState)?;
        let data = match packet_tag {
            PacketTag::Subscribe => PacketData::Subscribe(SubscribePacket::deserialize(&mut bytes)?),
            PacketTag::Unsubscribe => PacketData::Unsubscribe(UnsubscribePacket::deserialize(&mut bytes)?),
            PacketTag::Publish => PacketData::Publish(PublishPacket::deserialize(&mut bytes)?),
            PacketTag::Broadcast => PacketData::Broadcast(BroadcastPacket::deserialize(&mut bytes)?),
            PacketTag::ClientInputBroker => PacketData::ClientInputBroker(ClientInputBrokerPacket::deserialize(&mut bytes)?),
            PacketTag::ClientHello => PacketData::ClientHello(ClientHelloPacket::deserialize(&mut bytes)?),
            PacketTag::ClientHandshake => PacketData::ClientHandshake(ClientHandshakePacket::deserialize(&mut bytes)?),
            PacketTag::ClientInputShard => PacketData::ClientInputShard(ClientInputShardPacket::deserialize(&mut bytes)?),

            PacketTag::AllocateShards => PacketData::AllocateShards(AllocateShardsPacket::deserialize(&mut bytes)?),
            PacketTag::DeAllocateShards => PacketData::DeAllocateShards(DeAllocateShardsPacket::deserialize(&mut bytes)?),
            PacketTag::ShardCreation => PacketData::ShardCreation(ShardCreationPacket::deserialize(&mut bytes)?),
            PacketTag::ShardDestruction => PacketData::ShardDestruction(ShardDestructionPacket::deserialize(&mut bytes)?),
            PacketTag::AuthoritySwitch => PacketData::AuthoritySwitch(AuthoritySwitchPacket::deserialize(&mut bytes)?),

            PacketTag::OrchestratorHello => PacketData::OrchestratorHello(OrchestratorHelloPacket::deserialize(&mut bytes)?),
            PacketTag::Heartbeat => PacketData::Heartbeat(HeartbeatPacket::deserialize(&mut bytes)?),
        };
        Ok(Self { tag, data })
    }
    fn write(self) -> Result<bytes::Bytes, SerializationError> {
        let mut buffer = bytes::BytesMut::new();
        self.tag.serialize(&mut buffer)?;
        match self.data {
            PacketData::Subscribe(data) => data.serialize(&mut buffer)?,
            PacketData::Unsubscribe(data) => data.serialize(&mut buffer)?,
            PacketData::Publish(data) => data.serialize(&mut buffer)?,
            PacketData::Broadcast(data) => data.serialize(&mut buffer)?,
            PacketData::ClientInputBroker(data) => data.serialize(&mut buffer)?,
            PacketData::ClientHello(data) => data.serialize(&mut buffer)?,
            PacketData::ClientHandshake(data) => data.serialize(&mut buffer)?,
            PacketData::ClientInputShard(data) => data.serialize(&mut buffer)?,

            PacketData::AllocateShards(data) => data.serialize(&mut buffer)?,
            PacketData::DeAllocateShards(data) => data.serialize(&mut buffer)?,
            PacketData::ShardCreation(data) => data.serialize(&mut buffer)?,
            PacketData::ShardDestruction(data) => data.serialize(&mut buffer)?,
            PacketData::AuthoritySwitch(data) => data.serialize(&mut buffer)?,

            PacketData::OrchestratorHello(data) => data.serialize(&mut buffer)?,
            PacketData::Heartbeat(data) => data.serialize(&mut buffer)?,
        };
        Ok(buffer.freeze())
    }
}