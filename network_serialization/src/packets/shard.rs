use bevy::math::Vec2;
use bevy::reflect::erased_serde::__private::serde::{Deserialize, Serialize};
use bytes::{Bytes, BytesMut};
use crate::{Deserializable, Serializable, SerializationError};

#[derive(Debug, PartialEq, Clone)]
pub struct ClientInputShardPacket {
    pub client_id : u32,
    pub input: [u8; 16],
}
impl Serializable for ClientInputShardPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.client_id.serialize(bytes)?;
        self.input.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for ClientInputShardPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let client_id = u32::deserialize(bytes)?;
        let input = <[u8; 16]>::deserialize(bytes)?;
        Ok(Self { client_id, input })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct HandoffRequestPacket {
    pub entity_id : u32,
    pub pos: Vec2,
    pub vel: Vec2,
    pub state: [u8; 64],
}
impl Serializable for HandoffRequestPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.entity_id.serialize(bytes)?;
        self.pos.serialize(bytes)?;
        self.vel.serialize(bytes)?;
        self.state.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for HandoffRequestPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let entity_id = u32::deserialize(bytes)?;
        let pos = Vec2::deserialize(bytes)?;
        let vel = Vec2::deserialize(bytes)?;
        let state = <[u8; 64]>::deserialize(bytes)?;
        Ok(Self { entity_id, pos, vel, state })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct HandoffAcceptPacket {
    pub entity_id : u32,
}
impl Serializable for HandoffAcceptPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.entity_id.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for HandoffAcceptPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let entity_id = u32::deserialize(bytes)?;
        Ok(Self { entity_id })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct HandoffRejectPacket {
    pub entity_id : u32,
}
impl Serializable for HandoffRejectPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.entity_id.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for HandoffRejectPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let entity_id = u32::deserialize(bytes)?;
        Ok(Self { entity_id })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GhostUpdatePacket {
    pub entity_id : u32,
    pub pos: Vec2,
    pub vel: Vec2,
}
impl Serializable for GhostUpdatePacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.entity_id.serialize(bytes)?;
        self.pos.serialize(bytes)?;
        self.vel.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for GhostUpdatePacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let entity_id = u32::deserialize(bytes)?;
        let pos = Vec2::deserialize(bytes)?;
        let vel = Vec2::deserialize(bytes)?;
        Ok(Self { entity_id, pos, vel })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct HandoffCompletePacket {
    pub entity_id : u32,
}
impl Serializable for HandoffCompletePacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.entity_id.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for HandoffCompletePacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let entity_id = u32::deserialize(bytes)?;
        Ok(Self { entity_id })
    }
}