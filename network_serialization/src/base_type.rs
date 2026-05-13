use bytes::{Buf, BufMut, Bytes};
use ordered_float::NotNan;
use crate::{Deserializable, Serializable, SerializationError};

impl Serializable for NotNan<f32>
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        let bytes = serialize_f32(self);
        stream.put_slice(&bytes);

        Ok(())
    }
}

impl Deserializable for NotNan<f32>
{
    fn deserialize(bytes: &mut Bytes) -> Self
    {
        deserialize_f32(bytes)
    }
}

impl Serializable for f32
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        let not_nan_self = NotNan::new(self)?;
        not_nan_self.serialize(stream)
    }
}

impl Deserializable for f32
{
    fn deserialize(bytes: &mut Bytes) -> Self
    {
        let value = NotNan::deserialize(bytes);
        value.into_inner()
    }
}

impl Serializable for u8
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        stream.put_u8(self);
        Ok(())
    }
}

impl Deserializable for u8
{
    fn deserialize(bytes: &mut Bytes) -> Self
    {
        bytes.get_u8()
    }
}

fn serialize_f32(value: NotNan<f32>) -> [u8; 2]
{
    let value = value * 100.0;
    let value = value.trunc() as i16;
    value.to_be_bytes()
}

fn deserialize_f32(bytes: &mut bytes::Bytes) -> NotNan<f32>
{
    let value = bytes.get_i16();
    unsafe { NotNan::new_unchecked(value as f32 / 100.0) }
}
