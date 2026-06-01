use std::collections::HashMap;
use std::time;
use crate::geometry::prelude as geo;

pub struct ShardManager
{
    shards: HashMap<ShardId, Shard>,
    deleted_in_use: HashMap<ShardId, (time::Instant, Shard)>,
    replaced_shards: HashMap<ShardId, ShardId>,
    free_shards: HashMap<ShardId, time::Instant>,
}

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
        }
    }

    pub fn get_shard(&mut self, id: ShardId) -> Option<&mut Shard>
    {
    self.shards.get_mut(&id)
    }

    pub fn resolve_id(&mut self, id: ShardId) -> ShardId
    {
        let mut resolved_id = id;
        while let Some(new_id) = self.replaced_shards.get(&resolved_id)
        {
            resolved_id = *new_id;
        }

        resolved_id
    }

    pub fn new_shard(&mut self, authority_bounds: geo::Rect, subscribe_bounds: geo::Rect) -> Option<ShardId>
    {
        let shard_id = self.get_free_shard()?;

        self.free_shards.remove(&shard_id);
        self.shards.insert(shard_id, Shard::new(authority_bounds, subscribe_bounds));

        Some(shard_id)
    }

    // TODO : implement a function to clean up replaced shard

    pub fn on_receive_shard_creation(&mut self, shard_created: ShardId)
    {
        match self.get_deleted_in_use_shard()
        {
            None => {}
            Some(deleted_shard_id) =>
            {
                let (_, deleted_shard) = &self.deleted_in_use[&deleted_shard_id];

                self.shards.insert(shard_created, Shard::new(deleted_shard.authority_bound, deleted_shard.subscribe_bound));
                self.replaced_shards.insert(deleted_shard_id, shard_created);
                self.deleted_in_use.remove(&deleted_shard_id);
            }
        }
        self.free_shards.insert(shard_created, time::Instant::now());
    }

    pub fn on_receive_shard_deletion(&mut self, deleted_shard: ShardId)
    {
        match self.free_shards.remove(&deleted_shard)
        {
            Some(_) => return,
            None => (),
        }

        if     self.deleted_in_use.contains_key(&deleted_shard)
            || self.replaced_shards.contains_key(&deleted_shard)
        {
            eprintln!("Error: Double deletion on network, ignoring");
            return;
        }

        let shard = match self.shards.remove(&deleted_shard)
        {
            Some(shard) => shard,
            None =>
            {
                eprintln!("Error : deleted shard was not in any of our cache");
                return;
            },
        };

        eprintln!("Catastrophic error : shard in use was deleted");
        match self.new_shard(shard.authority_bound, shard.subscribe_bound)
        {
            Some(new_shard) =>
            {
                println!("We were able to recover using another shard");
                self.replaced_shards.insert(deleted_shard, new_shard);
            }
            None =>
            {
                println!("Requesting another shard created");
                self.deleted_in_use.insert(deleted_shard, (time::Instant::now(), shard));
                // TODO : request new shard on the network
                return;
            }
        };
    }

    fn get_free_shard(&self) -> Option<ShardId>
    {
        self.shards.keys().copied().next()
    }

    fn get_deleted_in_use_shard(&self) -> Option<ShardId>
    {
        self.deleted_in_use.keys().copied().next()
    }
}

pub struct Shard
{
    entities_count: usize,
    authority_bound: geo::Rect,
    subscribe_bound: geo::Rect,
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
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct ShardId(u64);
