use std::net::Ipv4Addr;
use bytes::{Buf, BufMut};
use crate::{Deserializable, Serializable, SerializationError};

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