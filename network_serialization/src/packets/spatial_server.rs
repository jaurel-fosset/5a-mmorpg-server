use crate::*;
use bytes::{Bytes, BytesMut};
use std::net::Ipv6Addr;

#[derive(Debug)]
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
    pub fn shard_count(&self) -> u64 { self.shard_count }
}

impl Serializable for AllocateShardsPacket
{
    fn serialize(self, stream: &mut BytesMut) -> Result<(), SerializationError>
    {
        self.shard_count.serialize(stream)
    }
}

impl Deserializable for AllocateShardsPacket
{
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let shard_count = u64::deserialize(bytes)?;
        Ok(Self { shard_count })
    }
}

#[derive(Debug)]
pub struct DeAllocateShardsPacket
{
    shards: Vec<Ipv6Addr>,
}

impl Serializable for DeAllocateShardsPacket
{
    fn serialize(self, stream: &mut BytesMut) -> Result<(), SerializationError>
    {
        self.shards.serialize(stream)
    }
}

impl Deserializable for DeAllocateShardsPacket
{
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let shards = Vec::deserialize(bytes)?;
        Ok(Self { shards })
    }
}

#[derive(Debug)]
pub struct ShardCreationPacket
{
    pub shards: Vec<Ipv6Addr>,
}

impl Serializable for ShardCreationPacket
{
    fn serialize(self, stream: &mut BytesMut) -> Result<(), SerializationError>
    {
        self.shards.serialize(stream)
    }
}

impl Deserializable for ShardCreationPacket
{
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let shards = Vec::deserialize(bytes)?;
        Ok(Self { shards })
    }
}

#[derive(Debug)]
pub struct ShardDestructionPacket
{
    pub shard: Ipv6Addr,
}

impl Serializable for ShardDestructionPacket
{
    fn serialize(self, stream: &mut BytesMut) -> Result<(), SerializationError>
    {
        self.shard.serialize(stream)
    }
}

impl Deserializable for ShardDestructionPacket
{
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let shard = Ipv6Addr::deserialize(bytes)?;
        Ok(Self { shard })
    }
}

#[derive(Debug)]
pub struct AuthoritySwitchPacket
{
    old_shard: u32,
    new_shard: u32,
    client: u32,
}

impl AuthoritySwitchPacket
{
    pub fn new(old_shard: u32, new_shard: u32, client: u32) -> Self
    {
        Self { old_shard, new_shard, client }
    }
}

impl Serializable for AuthoritySwitchPacket
{
    fn serialize(self, stream: &mut BytesMut) -> Result<(), SerializationError>
    {
        self.old_shard.serialize(stream)?;
        self.new_shard.serialize(stream)?;
        self.client.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for AuthoritySwitchPacket
{
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let old_shard = u32::deserialize(bytes)?;
        let new_shard = u32::deserialize(bytes)?;
        let client = u32::deserialize(bytes)?;

        Ok(Self { old_shard, new_shard, client })
    }
}
