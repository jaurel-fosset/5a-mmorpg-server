use std::net::{Ipv4Addr, Ipv6Addr};
use bytes::{Buf, BufMut, Bytes};
use crate::{Deserializable, NetworkId, Serializable, SerializationError};

impl Serializable for Ipv4Addr
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        stream.put_slice(self.octets().as_ref());
        Ok(())
    }
}

impl Deserializable for Ipv4Addr
{
    fn deserialize(stream: &mut bytes::Bytes) -> Result<Self, SerializationError>
    {
        let buffer = [stream.get_u8(), stream.get_u8(), stream.get_u8(), stream.get_u8()];
        Ok(Ipv4Addr::new(buffer[0], buffer[1], buffer[2], buffer[3]))
    }
}

// TODO : implement Serialization for Ipv6Addr

impl Serializable for NetworkId
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        self.0.serialize(stream)
    }
}

impl Deserializable for NetworkId
{
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let id = u64::deserialize(bytes)?;
        Ok(Self(id))
    }
}