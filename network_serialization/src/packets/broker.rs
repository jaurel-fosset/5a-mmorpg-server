use crate::input::InputData;
use crate::packets::topic::TopicTree;
use crate::{Deserializable, Serializable, SerializationError};
use bytes::{Bytes, BytesMut};

#[derive(Debug, PartialEq, Clone)]
pub struct SubscribePacket{
    pub client_id: u32,
    pub topic: TopicTree,
}
impl Serializable for SubscribePacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.client_id.serialize(bytes)?;
        self.topic.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for SubscribePacket {
    fn deserialize(mut bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let client_id = u32::deserialize(&mut bytes)?;
        let topic = TopicTree::deserialize(&mut bytes)?;
        Ok(Self { client_id, topic })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnsubscribePacket{
    pub client_id: u32,
    pub topic: TopicTree,
}
impl Serializable for UnsubscribePacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.client_id.serialize(bytes)?;
        self.topic.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for UnsubscribePacket {
    fn deserialize(mut bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let client_id = u32::deserialize(&mut bytes)?;
        let topic = TopicTree::deserialize(&mut bytes)?;
        Ok(Self { client_id, topic })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PublishPacket{
    pub data: Vec<TopicTree>,
}
impl Serializable for PublishPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.data.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for PublishPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let topic = Vec::<TopicTree>::deserialize(bytes)?;
        Ok(Self { data: topic })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BroadcastPacket{
    pub data: Vec<TopicTree>,
}
impl Serializable for BroadcastPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.data.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for BroadcastPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let topic = Vec::<TopicTree>::deserialize(bytes)?;
        Ok(Self { data: topic })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClientInputBrokerPacket {
    pub inputs: [InputData; 16],
}
impl Serializable for ClientInputBrokerPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.inputs.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for ClientInputBrokerPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let input = <[InputData; 16]>::deserialize(bytes)?;
        Ok(Self { inputs: input, })
    }
}

#[repr(u8)]
#[derive(int_enum::IntEnum, Debug, Eq, PartialEq, Clone, Copy)]
pub enum NetworkId {
    Orchestrator = 0,
    Broker,
    Spatial,
    Shard,
    Client,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClientHelloPacket {
    pub client_type: NetworkId,
}
impl Serializable for ClientHelloPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        (self.client_type as u8).serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for ClientHelloPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let data = u8::deserialize(bytes)?;
        let client_type = NetworkId::try_from(data)
            .map_err(|_| SerializationError::InvalidDeserializationState)?;
        Ok(Self {client_type})
    }
}

#[derive(Debug)]
pub struct ClientHandshakePacket {
    pub client_id: u32,
}
impl Serializable for ClientHandshakePacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.client_id.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for ClientHandshakePacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let client_id = u32::deserialize(bytes)?;
        Ok(Self { client_id })
    }
}