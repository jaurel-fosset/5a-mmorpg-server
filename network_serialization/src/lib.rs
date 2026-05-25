use bytes::{Bytes, TryGetError};
use thiserror::Error;

#[cfg(feature = "bevy_support")]
pub mod bevy;
pub mod base_type;
pub mod packets;
pub mod net;
#[cfg(test)]
mod tests;

pub trait Serializable
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>;
}

pub trait Deserializable
{
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> where Self: Sized;
}

#[derive(Error, Debug, Copy, Clone)]
pub enum SerializationError
{
    #[error("The value is in a state preventing its serialization")]
    NotSerializableState,
    #[error("Not enough bits in buffer to make the value")]
    NotEnoughBits,
    #[error("Invalid deserialization state")]
    InvalidDeserializationState,
}

impl From<TryGetError> for SerializationError
{
    fn from(_: TryGetError) -> Self { Self::NotEnoughBits }
}