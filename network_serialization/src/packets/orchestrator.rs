use std::net::Ipv6Addr;
use bytes::{Bytes, BytesMut};
use crate::packets::Packet;
use crate::*;

pub struct HelloPacket
{
    pub orchestrator: Ipv6Addr,
    pub redis_dns: Ipv6Addr,
}

impl Serializable for HelloPacket
{
    fn serialize(self, stream: &mut BytesMut) -> Result<(), SerializationError>
    {
        self.orchestrator.serialize(stream)?;
        self.redis_dns.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for HelloPacket
{
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let orchestrator = Ipv6Addr::deserialize(bytes)?;
        let redis_dns = Ipv6Addr::deserialize(bytes)?;

        Ok(Self { orchestrator, redis_dns })
    }
}