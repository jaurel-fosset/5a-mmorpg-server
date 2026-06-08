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

pub struct AuthoritySwitchPacket
{
    old_shard: Ipv6Addr,
    new_shard: Ipv6Addr,
    client: Ipv6Addr
}

impl AuthoritySwitchPacket
{
    pub fn new(old_shard: Ipv6Addr, new_shard: Ipv6Addr, client: Ipv6Addr) -> Self
    {
        Self { old_shard, new_shard, client }
    }
}

impl Packet for AuthoritySwitchPacket
{
    fn read(mut bytes: Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let old_shard = Ipv6Addr::deserialize(&mut bytes)?;
        let new_shard = Ipv6Addr::deserialize(&mut bytes)?;
        let client = Ipv6Addr::deserialize(&mut bytes)?;
        
        Ok(Self { old_shard, new_shard, client })
    }

    fn write(self) -> Result<Bytes, SerializationError>
    {
        let mut buffer = BytesMut::new();
        self.old_shard.serialize(&mut buffer)?;
        self.new_shard.serialize(&mut buffer)?;
        self.client.serialize(&mut buffer)?;

        Ok(buffer.freeze())
    }
}
