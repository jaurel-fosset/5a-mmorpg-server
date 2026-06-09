use bytes::{Buf, BufMut, BytesMut};
use crate::{Deserializable, Serializable, SerializationError};

impl Serializable for f32
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        stream.put_f32(self);
        Ok(())
    }
}

impl Deserializable for f32
{
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        Ok(bytes.try_get_f32()?)
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

impl Serializable for i8
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        stream.put_i8(self);
        Ok(())
    }
}

impl Deserializable for i8
{
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        Ok(bytes.try_get_i8()?)
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

impl Serializable for i16
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        stream.put_i16(self);
        Ok(())
    }
}

impl Deserializable for i16
{
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        Ok(bytes.try_get_i16()?)
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

impl Serializable for i32
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        stream.put_i32(self);
        Ok(())
    }
}

impl Deserializable for i32
{
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        Ok(bytes.try_get_i32()?)
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

impl Serializable for String
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        let bytes = self.as_bytes();
        stream.put_u32(bytes.len() as u32);
        stream.put_slice(bytes);
        Ok(())
    }
}

impl Deserializable for String
{
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        let len = bytes.try_get_u32()? as usize;
        let slice = bytes.copy_to_bytes(len);
        String::from_utf8(slice.to_vec()).map_err(|e| { SerializationError::InvalidDeserializationState })
    }
}

impl<T: Serializable, const N: usize> Serializable for [T; N] {
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError> {
        for element in self {
            element.serialize(stream)?;
        }
        Ok(())
    }
}

impl<T: Deserializable, const N: usize> Deserializable for [T; N] {
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError> {
        let mut vec = Vec::with_capacity(N);
        for _ in 0..N {
            vec.push(T::deserialize(bytes)?);
        }
        vec.try_into().map_err(|_| SerializationError::InvalidDeserializationState)
    }
}

impl<T: Serializable> Serializable for Vec<T> {
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError> {
        (self.len() as u16).serialize(stream)?;
        for element in self {
            element.serialize(stream)?;
        }
        Ok(())
    }
}

impl<T: Deserializable> Deserializable for Vec<T> {
    fn deserialize(bytes: &mut bytes::Bytes) -> Result<Self, SerializationError> {
        let len = u16::deserialize(bytes)? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::deserialize(bytes)?);
        }
        Ok(vec)
    }
}
