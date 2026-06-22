use crate::geometry::prelude as geo;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time;
use thiserror;
use crate::network_object::entity::{Entity, EntityId};

pub struct ShardManager
{
    pub(crate) shards: HashMap<ShardId, Shard>,
    deleted_in_use: HashMap<ShardId, (time::Instant, Shard)>,
    replaced_shards: HashMap<ShardId, ShardId>,
    pub(crate) free_shards: HashMap<ShardId, time::Instant>,
    shard_to_entity: HashMap<ShardId, Vec<EntityId>>,
}

const MAX_IDLE_TIME: time::Duration = time::Duration::from_secs(10);

impl ShardManager
{
    pub fn new() -> Self
    {
        ShardManager
        {
            shards: HashMap::new(),
            deleted_in_use: HashMap::new(),
            replaced_shards: HashMap::new(),
            free_shards: HashMap::new(),
            shard_to_entity: HashMap::new(),
        }
    }

    pub fn get_entity_count(&self, id: ShardId) -> Result<usize, ShardManagerError>
    {
        self.get_shard_resolved(id)
            .map(|shard| shard.entities_count)
    }

    pub fn in_subscribe_range(&mut self, id: ShardId, position: geo::Position) -> Result<bool, ShardManagerError>
    {
        self.get_shard_resolved(id)
            .map(|shard| shard.in_subscribe_range(position))
    }

    fn get_shard_resolved(&self, id: ShardId) -> Result<&Shard, ShardManagerError>
    {
        if let Some(shard) = self.shards.get(&id)
        {
            return Ok(shard);
        }

        match self.resolve_id(id)
        {
            Some(new_id) =>
                {
                    if self.shards.contains_key(&new_id)
                    {
                        Err(ShardManagerError::ShardReplaced(new_id))
                    } else { Err(ShardManagerError::ShardNotFound) }
                }
            None => Err(ShardManagerError::ShardNotFound),
        }
    }

    pub fn update_shard(&mut self, id: ShardId, authority_bounds: geo::Rect, load: usize) -> Option<()>
    {
        let shard = self.shards.get_mut(&id).unwrap();
        shard.entities_count = load;
        shard.authority_bound = authority_bounds;
        shard.subscribe_bound = authority_bounds.subscribe_rect();

        Some(())
    }

    pub fn should_resolve(&self, id: ShardId) -> bool
    {
        self.shards.contains_key(&id)
    }

    pub fn resolve_id(&self, id: ShardId) -> Option<ShardId>
    {
        let mut resolved_id = self.replaced_shards.get(&id)?;
        while let Some(new_id) = self.replaced_shards.get(resolved_id)
        {
            resolved_id = new_id;
        }

        Some(*resolved_id)
    }

    pub fn new_shard(&mut self, authority_bounds: geo::Rect) -> Option<ShardId>
    {
        let shard_id = self.get_free_shard()?;
        let subscribe_bound = authority_bounds.subscribe_rect();

        self.free_shards.remove(&shard_id);
        self.shards.insert(shard_id, Shard::new(authority_bounds, subscribe_bound));

        Some(shard_id)
    }

    pub fn release_shard(&mut self, shard_id: ShardId)
    {
        self.shards.remove(&shard_id);
        self.free_shards.insert(shard_id, time::Instant::now());
    }

    pub fn new_shard_with_capacity(&mut self, authority_bounds: geo::Rect, subscribe_bounds: geo::Rect, capacity: usize) -> Option<ShardId>
    {
        let shard_id = self.get_free_shard()?;

        self.free_shards.remove(&shard_id);

        let shard = Shard
        {
            entities_count: capacity,
            authority_bound: authority_bounds,
            subscribe_bound: subscribe_bounds,
        };
        self.shards.insert(shard_id, shard);

        Some(shard_id)
    }

    pub fn clean_up_replaced(&mut self, shard_id: ShardId)
    {
        self.replaced_shards.remove(&shard_id);
    }

    pub fn clean_up_free_shards(&mut self)
    {
        self.free_shards
            .retain(|_, instant|
                {
                    *instant - time::Instant::now() < MAX_IDLE_TIME
                });
    }

    pub fn on_receive_shard_creation(&mut self, shard_address: u32)
    {
        let shard_id = ShardId::new(shard_address);

        match self.get_deleted_in_use_shard()
        {
            None =>
                {
                    self.free_shards.insert(shard_id, time::Instant::now());
                    println!("Shard id {} inserted in free list", shard_id);
                }
            Some(deleted_shard_id) =>
                {
                    let (_, deleted_shard) = &self.deleted_in_use[&deleted_shard_id];

                    self.shards.insert(shard_id, Shard::new(deleted_shard.authority_bound, deleted_shard.subscribe_bound));
                    self.replaced_shards.insert(deleted_shard_id, shard_id);
                    self.deleted_in_use.remove(&deleted_shard_id);
                }
        }
    }

    pub fn on_receive_shard_deletion(&mut self, deleted_shard: u32) -> Result<(), DeletedShardUnrecovered>
    {
        let deleted_shard = ShardId::new(deleted_shard);

        if let Some(_) = self.free_shards.remove(&deleted_shard)
        {
            return Ok(());
        }

        if     self.deleted_in_use.contains_key(&deleted_shard)
            || self.replaced_shards.contains_key(&deleted_shard)
        {
            eprintln!("Error: Double deletion on network, ignoring");
            return Ok(());
        }

        let shard = match self.shards.remove(&deleted_shard)
        {
            Some(shard) => shard,
            None => return Ok(()),
        };

        eprintln!("Catastrophic error : shard in use was deleted");
        match self.new_shard(shard.authority_bound)
        {
            Some(new_shard) =>
            {
                println!("We were able to recover using another shard");
                self.replaced_shards.insert(deleted_shard, new_shard);
                Ok(())
            }
            None =>
            {
                println!("Requesting another shard created");
                self.deleted_in_use.insert(deleted_shard, (time::Instant::now(), shard));
                Err(DeletedShardUnrecovered)
            }
        }
    }

    fn get_free_shard(&self) -> Option<ShardId>
    {
        match self.free_shards.keys().copied().next() {
            Some(shard_id) => {
                println!("Shard id {} pour free_shard", shard_id);
                Some(shard_id)
            },
            None => {
                println!("No shard ids available");
                None
            }
        }
    }

    fn get_deleted_in_use_shard(&self) -> Option<ShardId>
    {
        self.deleted_in_use.keys().copied().next()
    }

    pub fn reset_shards_load(&mut self) {
        for shard in self.shards.values_mut() {
            shard.entities_count = 0;
        }
    }

    pub fn increment_shard_load(&mut self, shard_id: ShardId) {
        if let Some(shard) = self.shards.get_mut(&shard_id){
            shard.entities_count += 1;
        }
    }

    pub fn get_entities(&self, shard_id: ShardId) -> Option<&[EntityId]> {
        self.shard_to_entity.get(&shard_id)
            .map(|entities| { entities.as_slice() })
    }

    pub fn add_entity(&mut self, shard_id: ShardId, entity: EntityId)
    {
        self.shard_to_entity.entry(shard_id).or_insert(Vec::new()).push(entity);
    }

    pub fn remove_entity(&mut self, shard_id: ShardId, entity: EntityId)
    {
        self.shard_to_entity.entry(shard_id).or_insert(Vec::new()).retain(|e| e != &entity)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Unable to recover from the deletion")]
pub struct DeletedShardUnrecovered;

pub struct Shard
{
    pub entities_count: usize,
    pub authority_bound: geo::Rect,
    pub subscribe_bound: geo::Rect,
}

impl Shard
{
    fn new(authority_bound: geo::Rect, subscribe_bound: geo::Rect) -> Self
    {
        Self
        {
            entities_count: 0,
            authority_bound,
            subscribe_bound,
        }
    }

    pub fn in_subscribe_range(&self, position: geo::Position) -> bool
    {
        position.overlap_rect(self.subscribe_bound)
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct ShardId(u32);

impl ShardId
{
    fn new(ip: u32) -> Self
    {
        ShardId(ip)
    }

    pub fn id(&self) -> u32 { self.0 }
}

impl Display for ShardId
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result
    {
        write!(formatter, "ShardId({})", self.0)
    }
}


#[derive(thiserror::Error, Debug)]
pub enum ShardManagerError
{
    #[error("Shard was not found")]
    ShardNotFound,
    #[error("Shard was destroyed and replaced by {0}")]
    ShardReplaced(ShardId),
}