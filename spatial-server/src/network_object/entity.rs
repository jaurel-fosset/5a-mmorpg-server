use crate::geometry::prelude as geo;
use crate::network_object::shard::{ShardId, ShardManager};
use std::collections::{HashMap, HashSet};
use network_serialization::packets::topic::{TopicTree, TopicTreeType};
use crate::network_connection::NetworkGlobalState;

#[derive(Debug,Clone)]
pub struct EntityManager
{
    entities: HashMap<EntityId,Entity>,
}

impl EntityManager
{
    pub fn new() -> Self
    {
        EntityManager { entities: HashMap::new() }
    }

    pub fn receive_new_entities<T>(&mut self, network_manager: &mut NetworkGlobalState, shard_manager: &mut ShardManager, entities: T)
    where
        T: IntoIterator<Item=(EntityId, geo::Position, ShardId)>
    {
        let entities = entities.into_iter()
            .map(|(entity_id, pos, shard_id)| {
                (entity_id, Entity::new(entity_id, pos, shard_id))
            });

        for (id,entity) in entities {
            if !self.entities.contains_key(&id) {
                let mut input = TopicTree::new_empty("input".to_string());
                input.add_leaf(id.0.to_string(), Vec::new());

                let mut entities = TopicTree::new_empty("entities".to_string());
                entities.add_tree(input);

                match network_manager.subscribe(entity.current_shard.id(), entities) {
                    Ok(_) => (),
                    Err(_) => (),
                };

                on_entity_join_shard(network_manager, shard_manager, id, entity.current_shard);

                /*if let Some(entities) = shard_manager.get_entities(entity.current_shard) {
                    for entity_ in entities{
                        let mut tree_entities = TopicTree::new_empty("entities".to_string());
                        let mut positions = TopicTree::new_empty("position".to_string());

                        positions.add_leaf(format!("{}", entity.id.0), Vec::new());

                        tree_entities.add_tree(positions);

                        match network_manager.subscribe(entity.id().0, tree_entities)
                        {
                            Ok(_) => (),
                            Err(_) => (),
                        }
                    }
                }*/

                self.entities.insert(id,entity);
            } else {
                let Some(_entity) = self.entities.get_mut(&id) else { continue; };
                _entity.position = entity.position;

                /*if _entity.current_shard != entity.current_shard {
                    if let Some(entities) = shard_manager.get_entities(entity.current_shard) {
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
                    }

                    if let Some(entities) = shard_manager.get_entities(entity.current_shard) {
                        for entity_ in entities{
                            let mut tree_entities = TopicTree::new_empty("entities".to_string());
                            let mut positions = TopicTree::new_empty("position".to_string());

                            positions.add_leaf(format!("{}", entity.id.0), Vec::new());

                            tree_entities.add_tree(positions);

                            match network_manager.subscribe(entity.id().0, tree_entities)
                            {
                                Ok(_) => (),
                                Err(_) => (),
                            }
                        }
                    }
                }*/

            }
        }
    }

    pub fn entities(&mut self) -> impl Iterator<Item=&Entity>
    {
        self.entities.values()
    }

    pub fn entities_mut(&mut self) -> impl Iterator<Item=&mut Entity>
    {
        self.entities.values_mut()
    }
}


pub fn on_entity_join_shard(
    network: &mut NetworkGlobalState,
    shard_manager: &ShardManager,
    new_entity: EntityId,
    shard_id: ShardId,
) {
    let entities_in_shard = match shard_manager.get_entities(shard_id) {
        Some(entities) => entities.to_vec(),
        None => return,
    };

    let mut positions = TopicTree::new_empty("position".to_string());
    for other in &entities_in_shard {
        positions.add_leaf(other.0.to_string(), Vec::new());
    }

    if let TopicTreeType::Node(ref node) = positions.item {
        if !node.data.is_empty() {
            let mut entities_tree = TopicTree::new_empty("entities".to_string());
            entities_tree.add_tree(positions);
            network.subscribe(new_entity.0, entities_tree).ok();
        }
    }

    // Un message par autre entité pour les abonner à new_entity
    for other in &entities_in_shard {
        if *other == new_entity { continue; }
        network.subscribe(other.0, position_topic(new_entity.0)).ok();
    }
}

pub fn on_entity_leave_shard(
    network: &mut NetworkGlobalState,
    shard_manager: &ShardManager,
    leaving_entity: EntityId,
    shard_id: ShardId,
) {
    let entities_in_shard = match shard_manager.get_entities(shard_id) {
        Some(entities) => entities.to_vec(),
        None => return,
    };

    let mut positions = TopicTree::new_empty("position".to_string());
    for other in &entities_in_shard {
        positions.add_leaf(other.0.to_string(), Vec::new());
    }

    if let TopicTreeType::Node(ref node) = positions.item {
        if !node.data.is_empty() {
            let mut entities_tree = TopicTree::new_empty("entities".to_string());
            entities_tree.add_tree(positions);
            network.unsubscribe(leaving_entity.0, entities_tree).ok();
        }
    }

    // Un message par autre entité pour les abonner à new_entity
    for other in &entities_in_shard {
        if *other == leaving_entity { continue; }
        network.unsubscribe(other.0, position_topic(leaving_entity.0)).ok();
    }
}
pub fn position_topic(entity_id: u32) -> TopicTree {
    let mut positions = TopicTree::new_empty("position".to_string());
    positions.add_leaf(entity_id.to_string(), Vec::new());
    let mut entities = TopicTree::new_empty("entities".to_string());
    entities.add_tree(positions);
    entities
}

#[derive(Debug, Clone)]
pub struct Entity
{
    id: EntityId,
    position: geo::Position,
    current_shard: ShardId,
    subscribed_shard: HashSet<ShardId>,
}

impl Entity
{
    fn new(id: EntityId, position: geo::Position, current_shard: ShardId) -> Self
    {
        Self
        {
            id,
            position,
            current_shard,
            subscribed_shard: HashSet::new(),
        }
    }

    pub fn id(&self) -> EntityId
    {
        self.id
    }

    pub fn position(&self) -> &geo::Position
    {
        &self.position
    }

    pub fn current_shard(&self) -> ShardId
    {
        self.current_shard
    }

    pub fn update_subscription(&mut self, shards: HashSet<ShardId>) -> (Vec<ShardId>, Vec<ShardId>)
    {
        let added = shards.difference(&self.subscribed_shard)
            .copied().collect::<Vec<_>>();
        let removed = self.subscribed_shard.difference(&shards)
            .copied().collect::<Vec<_>>();
        self.subscribed_shard = shards;

        (added, removed)
    }

    pub fn switch_current_shard(&mut self, new_shard: ShardId)
    {
        self.current_shard = new_shard;
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct EntityId(pub u32);

