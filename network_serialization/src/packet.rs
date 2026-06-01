use crate::packets::Packet;
use crate::{Deserializable, Serializable, SerializationError};
use crate::packets::broker::{BroadcastPacket, ClientInputBrokerPacket, PublishPacket, RegisterPlayerPacket, SubscribePacket, UnsubscribePacket};
use crate::packets::shard::{ClientInputShardPacket, GhostUpdatePacket, HandoffAcceptPacket, HandoffCompletePacket, HandoffRejectPacket, HandoffRequestPacket};

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
    RegisterPlayer(RegisterPlayerPacket),

    ClientInputShard(ClientInputShardPacket),
    HandoffRequest(HandoffRequestPacket),
    HandoffAccept(HandoffAcceptPacket),
    HandoffReject(HandoffRejectPacket),
    GhostUpdate(GhostUpdatePacket),
    HandoffComplete(HandoffCompletePacket),
}

#[repr(u8)]
pub enum PacketTag {
    Subscribe = 0x00,
    Unsubscribe = 0x01,
    Publish = 0x02,
    Broadcast = 0x03,
    ClientInputBroker = 0x04,
    RegisterPlayer = 0x05,

    ClientInputShard = 0x06,

    HandoffRequest = 0x20,
    HandoffAccept = 0x21,
    HandoffReject = 0x22,
    GhostUpdate = 0x23,
    HandoffComplete = 0x24,
}

impl PacketData {
    pub fn tag(&self) -> u8 {
        match self {
            PacketData::Subscribe(_) => PacketTag::Subscribe as u8,
            PacketData::Unsubscribe(_) => PacketTag::Unsubscribe as u8,
            PacketData::Publish(_) => PacketTag::Publish as u8,
            PacketData::Broadcast(_) => PacketTag::Broadcast as u8,
            PacketData::ClientInputBroker(_) => PacketTag::ClientInputBroker as u8,
            PacketData::RegisterPlayer(_) => PacketTag::RegisterPlayer as u8,

            PacketData::ClientInputShard(_) => PacketTag::ClientInputShard as u8,
            PacketData::HandoffRequest(_) => PacketTag::HandoffRequest as u8,
            PacketData::HandoffAccept(_) => PacketTag::HandoffAccept as u8,
            PacketData::HandoffReject(_) => PacketTag::HandoffReject as u8,
            PacketData::GhostUpdate(_) => PacketTag::GhostUpdate as u8,
            PacketData::HandoffComplete(_) => PacketTag::HandoffComplete as u8,
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
            0x05 => Ok(PacketTag::RegisterPlayer),

            0x06 => Ok(PacketTag::ClientInputShard),
            0x20 => Ok(PacketTag::HandoffRequest),
            0x21 => Ok(PacketTag::HandoffAccept),
            0x22 => Ok(PacketTag::HandoffReject),
            0x23 => Ok(PacketTag::GhostUpdate),
            0x24 => Ok(PacketTag::HandoffComplete),

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
            PacketTag::RegisterPlayer => PacketData::RegisterPlayer(RegisterPlayerPacket::deserialize(&mut bytes)?),
            PacketTag::ClientInputShard => PacketData::ClientInputShard(ClientInputShardPacket::deserialize(&mut bytes)?),
            PacketTag::HandoffRequest => PacketData::HandoffRequest(HandoffRequestPacket::deserialize(&mut bytes)?),
            PacketTag::HandoffAccept => PacketData::HandoffAccept(HandoffAcceptPacket::deserialize(&mut bytes)?),
            PacketTag::HandoffReject => PacketData::HandoffReject(HandoffRejectPacket::deserialize(&mut bytes)?),
            PacketTag::GhostUpdate => PacketData::GhostUpdate(GhostUpdatePacket::deserialize(&mut bytes)?),
            PacketTag::HandoffComplete => PacketData::HandoffComplete(HandoffCompletePacket::deserialize(&mut bytes)?),
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
            PacketData::RegisterPlayer(data) => data.serialize(&mut buffer)?,
            PacketData::ClientInputShard(data) => data.serialize(&mut buffer)?,
            PacketData::HandoffRequest(data) => data.serialize(&mut buffer)?,
            PacketData::HandoffAccept(data) => data.serialize(&mut buffer)?,
            PacketData::HandoffReject(data) => data.serialize(&mut buffer)?,
            PacketData::GhostUpdate(data) => data.serialize(&mut buffer)?,
            PacketData::HandoffComplete(data) => data.serialize(&mut buffer)?
        };
        Ok(buffer.freeze())
    }
}