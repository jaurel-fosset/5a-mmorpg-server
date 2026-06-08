use crate::packets::Packet;
use crate::{Deserializable, Serializable, SerializationError};
use crate::packets::broker::{BroadcastPacket, ClientInputBrokerPacket, PublishPacket, ClientHelloPacket, SubscribePacket, UnsubscribePacket, ClientHandshakePacket};
use crate::packets::shard::ClientInputShardPacket;

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
}

#[repr(u8)]
pub enum PacketTag {
    Subscribe = 0x00,
    Unsubscribe = 0x01,
    Publish = 0x02,
    Broadcast = 0x03,
    ClientInputBroker = 0x04,
    ClientHello = 0x05,
    ClientHandshake = 0x10,

    ClientInputShard = 0x06,
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
        }
    }
}

impl TryFrom<u8> for PacketTag {
    type Error = SerializationError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(PacketTag::Subscribe),
            0x01 => Ok(PacketTag::Unsubscribe),
            0x02 => Ok(PacketTag::Publish),
            0x03 => Ok(PacketTag::Broadcast),
            0x04 => Ok(PacketTag::ClientInputBroker),
            0x05 => Ok(PacketTag::ClientHello),
            0x10 => Ok(PacketTag::ClientHandshake),
            0x06 => Ok(PacketTag::ClientInputShard),
            _ => Err(SerializationError::InvalidDeserializationState),
        }
    }
}

impl Packet for PacketMessage {
    fn read(mut bytes: bytes::Bytes) -> Result<Self, SerializationError> {
        let tag = u8::deserialize(&mut bytes)?;
        let packet_tag = PacketTag::try_from(tag)?;
        let data = match packet_tag {
            PacketTag::Subscribe => PacketData::Subscribe(SubscribePacket::deserialize(&mut bytes)?),
            PacketTag::Unsubscribe => PacketData::Unsubscribe(UnsubscribePacket::deserialize(&mut bytes)?),
            PacketTag::Publish => PacketData::Publish(PublishPacket::deserialize(&mut bytes)?),
            PacketTag::Broadcast => PacketData::Broadcast(BroadcastPacket::deserialize(&mut bytes)?),
            PacketTag::ClientInputBroker => PacketData::ClientInputBroker(ClientInputBrokerPacket::deserialize(&mut bytes)?),
            PacketTag::ClientHello => PacketData::ClientHello(ClientHelloPacket::deserialize(&mut bytes)?),
            PacketTag::ClientHandshake => PacketData::ClientHandshake(ClientHandshakePacket::deserialize(&mut bytes)?),
            PacketTag::ClientInputShard => PacketData::ClientInputShard(ClientInputShardPacket::deserialize(&mut bytes)?),
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
        };
        Ok(buffer.freeze())
    }
}