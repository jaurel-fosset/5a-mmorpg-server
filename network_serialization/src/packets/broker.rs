use bytes::{Bytes, BytesMut};
use crate::{Deserializable, Serializable, SerializationError};

#[derive(Debug, PartialEq, Clone)]
pub struct SubscribePacket{
    pub client_id: u32,
    pub topic: [u8; 32],
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
        let topic = <[u8; 32]>::deserialize(&mut bytes)?;
        Ok(Self { client_id, topic })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnsubscribePacket{
    pub client_id: u32,
    pub topic: [u8; 32],
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
        let topic = <[u8; 32]>::deserialize(&mut bytes)?;
        Ok(Self { client_id, topic })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PublishPacket{
    pub topic: [u8; 32],
    pub payload: Vec<u8>,
}
impl Serializable for PublishPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.topic.serialize(bytes)?;
        self.payload.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for PublishPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let topic = <[u8; 32]>::deserialize(bytes)?;
        let payload = Vec::<u8>::deserialize(bytes)?;
        Ok(Self { topic, payload })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BroadcastPacket{
    pub payload: Vec<u8>,
}
impl Serializable for BroadcastPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.payload.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for BroadcastPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let payload = Vec::<u8>::deserialize(bytes)?;
        Ok(Self { payload })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClientInputBrokerPacket {
    pub input: [u8; 16],
}
impl Serializable for ClientInputBrokerPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.input.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for ClientInputBrokerPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let input = <[u8; 16]>::deserialize(bytes)?;
        Ok(Self { input, })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RegisterPlayerPacket{}
impl Serializable for RegisterPlayerPacket {
    fn serialize(self, _bytes: &mut BytesMut) -> Result<(), SerializationError> {
        Ok(())
    }
}
impl Deserializable for RegisterPlayerPacket {
    fn deserialize(_bytes: &mut Bytes) -> Result<Self, SerializationError> {
        Ok(Self {})
    }
}