use std::net::{Ipv4Addr, Ipv6Addr};
use bytes::{Bytes, BytesMut};
use crate::packets::Packet;
use crate::*;

#[derive(Debug)]
pub struct OrchestratorHelloPacket
{
    pub orchestrator: Ipv4Addr,
    pub redis_dns: Ipv4Addr,
    pub broker: Ipv4Addr,
}

impl Serializable for OrchestratorHelloPacket
{
    fn serialize(self, stream: &mut BytesMut) -> Result<(), SerializationError>
    {
        self.orchestrator.serialize(stream)?;
        self.redis_dns.serialize(stream)?;
        self.broker.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for OrchestratorHelloPacket
{
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let orchestrator = Ipv4Addr::deserialize(bytes)?;
        let redis_dns = Ipv4Addr::deserialize(bytes)?;
        let broker = Ipv4Addr::deserialize(bytes)?;

        Ok(Self { orchestrator, redis_dns, broker })
    }
}