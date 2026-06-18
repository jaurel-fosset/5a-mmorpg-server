use crate::geometry::prelude as geo;
use crate::network_object::shard::ShardId;
use std::collections::{HashMap, HashSet};

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

    pub fn receive_new_entities<T>(&mut self, entities: T)
    where
        T: IntoIterator<Item=(EntityId, geo::Position, ShardId)>
    {
        let entities = entities.into_iter()
            .map(|(entity_id, pos, shard_id)| {
                (entity_id, Entity::new(entity_id, pos, shard_id))
            });
        self.entities.extend(entities);
    }

    pub fn entities(&mut self) -> impl Iterator<Item=Entity>
    {
        self.entities.values().cloned()
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

