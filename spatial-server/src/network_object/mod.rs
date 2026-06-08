use network_serialization::packets::Packet;
use crate::network_object::entity::Entity;
use crate::network_object::shard::ShardId;
use network_serialization::packets::spatial_server::*;
use crate::network_connection::SOCKET;

pub mod entity;
pub mod shard;

pub fn request_more_shards(amount: u64)
{
    // TODO : request new shard on the network
}

pub fn switch_authority(new_shard: ShardId, old_shard: ShardId, entity: &mut Entity)
{
    // TODO : send packet to notify new and old shard of the authority change
}