use std::net::Ipv6Addr;
use bytes::{Bytes, BytesMut};
use crate::*;
use crate::packets::Packet;

pub struct AllocateShardsPacket
{
    shard_count: u64,
}

impl AllocateShardsPacket
{
    pub fn new(shard_count: u64) -> AllocateShardsPacket
    {
        Self { shard_count }
    }
}

impl Packet for AllocateShardsPacket
{
    fn read(mut bytes: Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let shard_count = u64::deserialize(&mut bytes)?;
        Ok(Self { shard_count })
    }

    fn write(self) -> Result<Bytes, SerializationError>
    {
        let mut buffer = BytesMut::new();
        self.shard_count.serialize(&mut buffer)?;

        Ok(buffer.freeze())
    }
}

pub struct DeAllocateShardsPacket
{
    shards: Vec<Ipv6Addr>
}

impl Packet for DeAllocateShardsPacket
{
    fn read(mut bytes: Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let shards = Vec::deserialize(&mut bytes)?;
        Ok(Self { shards })
    }

    fn write(self) -> Result<Bytes, SerializationError>
    {
        let mut buffer = BytesMut::new();
        self.shards.serialize(&mut buffer)?;

        Ok(buffer.freeze())
    }
}

pub struct ShardCreationPacket
{
    shards: Vec<Ipv6Addr>
}

impl Packet for ShardCreationPacket
{
    fn read(mut bytes: Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let shards = Vec::deserialize(&mut bytes)?;
        Ok(Self { shards })
    }

    fn write(self) -> Result<Bytes, SerializationError>
    {
        let mut buffer = BytesMut::new();
        self.shards.serialize(&mut buffer)?;

        Ok(buffer.freeze())
    }
}

pub struct ShardDestructionPacket
{
    shard: Ipv6Addr,
}

impl Packet for ShardDestructionPacket
{
    fn read(mut bytes: Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let shard = Ipv6Addr::deserialize(&mut bytes)?;
        Ok(Self { shard })
    }

    fn write(self) -> Result<Bytes, SerializationError>
    {
        let mut buffer = BytesMut::new();
        self.shard.serialize(&mut buffer)?;

        Ok(buffer.freeze())
    }
}

pub struct AuthorityGainPacket
{
    client: NetworkId
}

impl Packet for AuthorityGainPacket
{
    fn read(mut bytes: Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let client = NetworkId::deserialize(&mut bytes)?;
        Ok(Self { client })
    }

    fn write(self) -> Result<Bytes, SerializationError>
    {
        let mut buffer = BytesMut::new();
        self.client.serialize(&mut buffer)?;

        Ok(buffer.freeze())
    }
}

pub struct AuthorityLostPacket
{
    client: NetworkId
}

impl Packet for AuthorityLostPacket
{
    fn read(mut bytes: Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let client = NetworkId::deserialize(&mut bytes)?;
        Ok(Self { client })
    }

    fn write(self) -> Result<Bytes, SerializationError>
    {
        let mut buffer = BytesMut::new();
        self.client.serialize(&mut buffer)?;

        Ok(buffer.freeze())
    }
}