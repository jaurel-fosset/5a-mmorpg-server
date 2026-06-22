use spatial_server::geometry::prelude as geo;
use spatial_server::network_connection::{NetworkEvent, NetworkGlobalState};
use spatial_server::network_object::entity::{on_entity_join_shard, on_entity_leave_shard, Entity, EntityId, EntityManager};
use spatial_server::network_object::shard::{ShardId, ShardManager};
use spatial_server::quad_tree::QuadTree;
use std::collections::{HashMap, HashSet};
use std::iter::Cloned;
use std::time::{Duration, Instant};
use network_serialization::packets::topic::TopicTree;

const MAX_AUTHORITY_SWITCH_RANGE: f32 = 100.0;


fn update_shard(network_manager: &mut NetworkGlobalState, quad_tree: &mut QuadTree, shard_manager: &mut ShardManager, entity: &mut Entity) {
    let shard_id = entity.current_shard();
    println!("entity {:?} est dans le shard {}",entity.id(), shard_id);
    shard_manager.increment_shard_load(shard_id);
}


fn update_subscription(network_manager: &mut NetworkGlobalState, quad_tree: &mut QuadTree, shard_manager: &mut ShardManager, entity_manager: &mut EntityManager)
{

    for entity in entity_manager.entities_mut()
    {
        let entity_position = *entity.position();

        let new_subscription = shard_in_subscribe_range(quad_tree, shard_manager, entity_position);
        let (added_shard, removed_shard) = entity.update_subscription(new_subscription);

        let mut tree_entities = TopicTree::new_empty("entities".to_string());
        let mut positions = TopicTree::new_empty("position".to_string());

        for shard in added_shard
        {
            let mut positions = TopicTree::new_empty("position".to_string());
            positions.add_leaf(format!("{}", entity.id().0), Vec::new());

            let mut tree_entities = TopicTree::new_empty("entities".to_string());
            tree_entities.add_tree(positions);

            match network_manager.subscribe(shard.id(), tree_entities)
            {
                Ok(_) => (),
                Err(_) => (),
            }
        }

        for shard in removed_shard
        {
            let mut positions = TopicTree::new_empty("position".to_string());
            positions.add_leaf(format!("{}", entity.id().0), Vec::new());

            let mut tree_entities = TopicTree::new_empty("entities".to_string());
            tree_entities.add_tree(positions);


            match network_manager.unsubscribe(shard.id(), tree_entities)
            {
                Ok(_) => (),
                Err(_) => (),
            }
        }
    }

fn handle_authority_switch(network_manager: &mut NetworkGlobalState, quad_tree: &mut QuadTree, shard_manager: &mut ShardManager, entity: &mut Entity)
{
    let entity_position = *entity.position();

    let current_shard = match quad_tree.shard_for(entity_position)
    {
        Some(shard) =>
        {
            shard_manager.resolve_id(shard).unwrap_or_else(|| shard)
        },
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
        /*if let Some(entities) = shard_manager.get_entities(previous_shard) {
            let mut tree_entities = TopicTree::new_empty("entities".to_string());
            let mut positions = TopicTree::new_empty("position".to_string());

            for entity_ in entities.to_owned() {
                positions.add_leaf(format!("{}", entity_.0), Vec::new());
                shard_manager.remove_entity(previous_shard, entity_);
            }
            tree_entities.add_tree(positions);

            match network_manager.unsubscribe(entity.id().0, tree_entities)
            {
                Ok(_) => (),
                Err(_) => (),
            }
        }*/


        on_entity_leave_shard(network_manager, &shard_manager, entity.id(), previous_shard);
        network_manager.switch_authority(current_shard, previous_shard, entity);
        entity.switch_current_shard(current_shard);
        on_entity_join_shard(network_manager, &shard_manager, entity.id(), current_shard);

        /*if let Some(entities) = shard_manager.get_entities(current_shard) {
            let mut tree_entities = TopicTree::new_empty("entities".to_string());
            let mut positions = TopicTree::new_empty("position".to_string());

            for entity_ in entities{
                positions.add_leaf(format!("{}", entity_.0), Vec::new());
            }
            tree_entities.add_tree(positions);

            match network_manager.subscribe(entity.id().0, tree_entities)
            {
                Ok(_) => (),
                Err(_) => (),
            }
        }*/




    } else {
        println!("current_shard == previous_shard for entity {}", entity.id().0);
    }
}

fn shard_in_subscribe_range(quad_tree: &mut QuadTree, shard_manager: &mut ShardManager, entity: geo::Position) -> HashSet<ShardId>
{
    let subscribe_range = geo::Circle
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
    let map_bounds = geo::Rect
    {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };

    let mut network = NetworkGlobalState::new();
    let mut shard_manager = ShardManager::new();
    let mut entity_manager = EntityManager::new();

    loop {
        network.poll_once();
        if network.is_orchestrator_connected() { break; }
    }

    let tick_duration = Duration::from_millis(66);

    let mut quad_tree: QuadTree = loop
    {
        let start_time = Instant::now();

        network.request_more_shards(1);
        if let Some(event) = network.poll_once()
        {
            match event
            {
                NetworkEvent::ShardsUpdate(created_shards, destroyed_shards) =>
                {
                    for shard in created_shards
                    {
                        shard_manager.on_receive_shard_creation(shard);
                    }
                    for shard in destroyed_shards
                    {
                        let mut lost_shard = 0;
                        match shard_manager.on_receive_shard_deletion(shard)
                        {
                            Ok(_) => (),
                            Err(_) => lost_shard += 1,
                        }

                        network.request_more_shards(lost_shard)
                    }
                }
                _ => (),
            }
        }

        match shard_manager.new_shard(map_bounds)
        {
            Some(shard) => break QuadTree::new(map_bounds, shard),
            None => (),
        }

        let work_duration = start_time.elapsed();
        if let Some(sleep_duration) = tick_duration.checked_sub(work_duration) {
            std::thread::sleep(sleep_duration);
        } else {
            println!("LAG: work took {}ms", work_duration.as_millis());
        }
    };

    println!("entrée boucle");

    loop
    {
        let start_time = Instant::now();

        if let Some(event) = network.poll_once()
        {
            match event
            {
                NetworkEvent::ShardsUpdate(created_shards, destroyed_shards) =>
                {
                    println!("Shards updated!");
                   
                    for shard in created_shards
                    {
                        shard_manager.on_receive_shard_creation(shard);
                    }
                    for shard in destroyed_shards
                    {
                        let mut lost_shard = 0;
                        match shard_manager.on_receive_shard_deletion(shard)
                        {
                            Ok(_) => (),
                            Err(_) => lost_shard += 1,
                        }

                        network.request_more_shards(lost_shard)
                    }
                }
                NetworkEvent::PositionUpdate(entity_positions) =>
                {
                    let positions = entity_positions
                        .into_iter()
                        .flat_map(|pos|
                        {
                            let entity_id = EntityId(pos.0);
                            let position = geo::Position::new(pos.1, pos.2);
                            let shard_id = quad_tree.shard_for(position)?;

                            Some((entity_id, position, shard_id))
                        });


                    for (entity_id, position, shard_id) in positions.clone() {
                        shard_manager.add_entity(shard_id,entity_id.into())
                    }

                    entity_manager.receive_new_entities(&mut network,&mut shard_manager,positions);


                    println!("entity_manager.entities: {:?}",entity_manager.entities().collect::<Vec<_>>());
                    
                    shard_manager.reset_shards_load();

                    for mut entity in entity_manager.entities_mut()
                    {
                        update_shard(&mut network, &mut quad_tree, &mut shard_manager, &mut entity);
                        //update_subscription(&mut network, &mut quad_tree, &mut shard_manager, &mut entity);
                        handle_authority_switch(&mut network, &mut quad_tree, &mut shard_manager, &mut entity);
                    }



                    update_subscription(&mut network, &mut quad_tree, &mut shard_manager, &mut entity_manager);

                    let vec_entity = entity_manager.entities().cloned().collect::<Vec<_>>();
                    let (shards_to_allocate, has_split_or_fused) = quad_tree
                        .split_and_fuse(&mut shard_manager, vec_entity.as_slice());

                    if shards_to_allocate != 0 {
                        network.request_more_shards(shards_to_allocate as u64);
                    }

                    if has_split_or_fused
                    {
                        for mut entity in entity_manager.entities_mut()
                        {
                            println!("avant le fuse {}",entity.current_shard());
                            handle_authority_switch(&mut network, &mut quad_tree, &mut shard_manager, &mut entity);
                            println!("après le fuse {}",entity.current_shard());
                        }
                    }
                }
            }
        }

        _ = network.send_heartbeat();

        let work_duration = start_time.elapsed();
        if let Some(sleep_duration) = tick_duration.checked_sub(work_duration) {
            //std::thread::sleep(sleep_duration);

        } else {
            println!("LAG: work took {}ms", work_duration.as_millis());
        }
    }
}
