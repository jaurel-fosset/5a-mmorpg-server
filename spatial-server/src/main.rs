use spatial_server::geometry::prelude as geo;
use spatial_server::geometry::prelude::*;
use spatial_server::network_connection::{NetworkEvent, NetworkGlobalState};
use spatial_server::network_object::entity::{Entity, EntityId, EntityManager};
use spatial_server::network_object::shard::{ShardId, ShardManager};
use spatial_server::quad_tree::QuadTree;
use std::collections::HashSet;

const MAX_AUTHORITY_SWITCH_RANGE: f32 = 100.0;


fn handle_authority_switch(network_manager: &mut NetworkGlobalState, quad_tree: &mut QuadTree, shard_manager: &mut ShardManager, entity: &mut Entity)
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
        network_manager.switch_authority(current_shard, previous_shard, entity);
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

fn main()
{
    let map_bounds = Rect
    {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };

    let mut network = NetworkGlobalState::new();
    let mut shard_manager = ShardManager::new();
    let mut entity_manager = EntityManager::new();

    network.request_more_shards(1);
    let mut quad_tree = loop
    {
        if let Some(event) = network.poll_once()
        {
            match event
            {
                NetworkEvent::ShardCreation(ip) =>
                    {
                        shard_manager.on_receive_shard_creation(ip);
                        let id = shard_manager.new_shard(map_bounds)
                            .unwrap();
                        break QuadTree::new(map_bounds, id);
                    }
                _ => (),
            }
        }
    };

    loop
    {
        if let Some(event) = network.poll_once()
        {
            match event
            {
                NetworkEvent::ShardCreation(ip) =>
                    {
                        shard_manager.on_receive_shard_creation(ip);
                    }
                NetworkEvent::ShardDestruction(ip) =>
                    {
                        let request_one_shard = shard_manager.on_receive_shard_deletion(ip);
                        if request_one_shard
                        {
                            network.request_more_shards(1);
                        }
                    }
                NetworkEvent::PositionUpdate(entity_positions) =>
                    {
                        let positions = entity_positions
                            .into_iter()
                            .map(|pos|
                                {
                                    let entity_id = EntityId(pos.0);
                                    let position = geo::Position::new(pos.1, pos.2);
                                    let shard_id = quad_tree.shard_for(position);

                                    (entity_id, position, shard_id)
                                })
                            .collect();

                        entity_manager.receive_new_entities(positions);

                        for entity in entity_manager.entities()
                        {
                            handle_authority_switch(&mut network, &mut quad_tree, &mut shard_manager, entity);
                        }

                        let shards_to_allocate = quad_tree
                            .split_and_fuse(&mut shard_manager, entity_manager.entities());

                        network.request_more_shards(shards_to_allocate as u64);
                    }
            }
        }
    }
}
