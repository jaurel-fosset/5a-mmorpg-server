use std::collections::HashSet;
use crate::geometry::prelude as geo;
use crate::network_object::shard::ShardId;

pub struct EntityManager
{
    entities: Vec<Entity>,
}

impl EntityManager
{
    pub fn new() -> Self
    {
        EntityManager { entities: Vec::new() }
    }
    
    pub fn receive_new_entities(&mut self, entities: &[(EntityId, geo::Position, ShardId)])
    {
        let entities = entities.into_iter()
            .map(|(entity_id, pos, shard_id)| {
                Entity::new(*entity_id, *pos, *shard_id)
            });
        self.entities.extend(entities);
    }
}

#[derive(Debug)]
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
    
    pub fn update_subscription(&mut self, shards: HashSet<ShardId>) -> Vec<ShardId>
    {
        let removed = self.subscribed_shard.difference(&shards)
            .copied().collect::<Vec<_>>();
        self.subscribed_shard = shards;
        removed
    }
    
    pub fn switch_current_shard(&mut self, new_shard: ShardId)
    {
        self.current_shard = new_shard;
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct EntityId(u64);

