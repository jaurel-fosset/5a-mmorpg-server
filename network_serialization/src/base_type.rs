use bytes::{Buf, BufMut};
use crate::{Deserializable, Serializable, SerializationError};

impl Serializable for f32
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        stream.put_slice(self.to_be_bytes().as_ref());
        Ok(())
    }
}

impl Deserializable for f32
{
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        bytes.try_get_u8()?;
        let bytes: [u8; 4] = [bytes[0], bytes[1], bytes[2], bytes[3]];
        Ok(f32::from_be_bytes(bytes))
    }
}

pub fn serialize_f32_with_precision_loss(value: f32, bytes: &mut bytes::BytesMut) -> Result<(), SerializationError>
{
    if !value.is_finite()
    {
        return Err(SerializationError::NotSerializableState);
    }

    let value = value * 100.0;
    let value = (value * i16::MAX as f32) / f32::MAX;
    let value = value.trunc() as i16;

    bytes.put_i16(value);
    Ok(())
}

pub fn deserialize_f32_with_precision_loss(bytes: &mut bytes::Bytes) -> Result<f32, SerializationError>
{
    let value = bytes.get_i16() as f32;
    let value = (value * f32::MAX) / i16::MAX as f32;
    Ok(value / 100.0)
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
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        Ok(bytes.try_get_u8()?)
    }
}

impl Serializable for u16
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        stream.put_u16(self);
        Ok(())
    }
}

impl Deserializable for u16
{
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        Ok(bytes.try_get_u16()?)
    }
}

impl Serializable for u32
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        stream.put_u32(self);
        Ok(())
    }
}

impl Deserializable for u32
{
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        Ok(bytes.try_get_u32()?)
    }
}

impl Serializable for u64
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        stream.put_u64(self);
        Ok(())
    }
}

impl Deserializable for u64
{
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        Ok(bytes.try_get_u64()?)
    }
}

impl Serializable for u128
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        stream.put_u128(self);
        Ok(())
    }
}

impl Deserializable for u128
{
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        Ok(bytes.try_get_u128()?)
    }
}
