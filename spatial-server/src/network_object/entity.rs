use crate::geometry::prelude as geo;
use crate::network_object::shard::ShardId;
use std::collections::{HashMap, HashSet};
use network_serialization::packets::topic::TopicTree;
use crate::network_connection::NetworkGlobalState;

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

    pub fn receive_new_entities<T>(&mut self, network_manager: &mut NetworkGlobalState,entities: T)
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
                self.entities.insert(id,entity);
            } else {
                let Some(_entity) = self.entities.get_mut(&id) else { continue; };
                _entity.position = entity.position;
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

