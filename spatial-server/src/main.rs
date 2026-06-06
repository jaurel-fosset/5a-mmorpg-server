use std::collections::HashSet;
use spatial_server::geometry::prelude::*;
use spatial_server::network_object::entity::Entity;
use spatial_server::network_object::{request_more_shards, switch_authority};
use spatial_server::network_object::shard::{ShardId, ShardManager};
use spatial_server::quad_tree::QuadTree;

const MAX_AUTHORITY_SWITCH_RANGE: f32 = 100.0;

fn update(quad_tree: &mut QuadTree, shard_manager: &mut ShardManager, entities: &mut [Entity])
{
    for entity in entities.iter_mut()
    {
        handle_authority_switch(quad_tree, shard_manager, entity);
    }

    let shards_to_allocate = quad_tree.split_and_fuse(shard_manager, entities);
    request_more_shards(shards_to_allocate as u64);
}

fn handle_authority_switch(quad_tree: &mut QuadTree, shard_manager: &mut ShardManager, entity: &mut Entity)
{
    let entity_position = *entity.position();

    let new_subscription = shard_in_subscribe_range(quad_tree, shard_manager, entity_position);
    entity.update_subscription(new_subscription);
    // TODO : send packet to actually subscribe and unsubscribe

    let current_shard = match quad_tree.shard_for(entity_position)
    {
        Some(shard) => shard,
        None =>
        {
            eprintln!("Error : an entity is out of bound. This is either because of a deleted\
            shard in use or the position is somehow really out of bounds");
            return;
        }
    };

    let previous_shard = entity.current_shard();

    if current_shard != previous_shard
    {
        switch_authority(current_shard, previous_shard, entity);
        entity.switch_current_shard(current_shard);
    }
}

fn shard_in_subscribe_range(quad_tree: &mut QuadTree, shard_manager: &mut ShardManager, entity: Position) -> HashSet<ShardId>
{
    let subscribe_range = Circle
    {
        center: entity,
        radius: MAX_AUTHORITY_SWITCH_RANGE,
    };

    quad_tree.shards_near(shard_manager, subscribe_range)
        .into_iter()
        .filter(|shard_id|
        {
            shard_manager.in_subscribe_range(*shard_id, entity)
                .expect("All the shard returned by the quad tree should be valid")
        })
        .collect()
}

fn main() {
    println!("Hello, world!");
}
