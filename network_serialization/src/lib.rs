use bytes::Bytes;
use ordered_float::FloatIsNan;

#[cfg(feature = "bevy_support")]
pub mod bevy;
pub mod base_type;

pub trait Packet
{
    fn read(bytes: Bytes) -> Self;
    fn write(self) -> bytes::Bytes;
}

pub trait Serializable
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>;
}

pub trait Deserializable
{
    fn deserialize(bytes: &mut Bytes) -> Self;
}

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