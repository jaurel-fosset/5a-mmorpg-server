use bytes::Bytes;
use ordered_float::FloatIsNan;

#[cfg(feature = "bevy_support")]
pub mod bevy;
pub mod base_type;
pub mod packets;


pub trait Serializable
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>;
}

pub trait Deserializable
{
    fn deserialize(bytes: &mut Bytes) -> Self;
}

#[derive(Debug, Copy, Clone)]
pub enum SerializationError
{
    BadArguments,
}

impl From<FloatIsNan> for SerializationError
{
    fn from(_: FloatIsNan) -> SerializationError
    {
        SerializationError::BadArguments
    }
}