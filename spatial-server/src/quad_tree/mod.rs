use crate::geometry::prelude::*;
use crate::network_object::entity::Entity;
use crate::network_object::shard::{ShardId, ShardManager, ShardManagerError};

pub struct QuadTree
{
    root: usize,
    nodes: Vec<QuadTreeNode>,
}

pub struct QuadTreeNode
{
    bounds: Rect,
    node_type: QuadTreeNodeType,
}

impl QuadTreeNode
{
    pub fn new_leaf(bounds: Rect, shard: ShardId) -> QuadTreeNode
    {
        Self
        {
            bounds,
            node_type: QuadTreeNodeType::Leaf(shard)
        }
    }
}

enum QuadTreeNodeType
{
    Node([usize; 4]),
    Leaf(ShardId),
}


const PLAYER_LIMIT: usize = 2;

impl QuadTree
{
    pub fn new(map_bounds: Rect, shard: ShardId) -> Self
    {
        Self
        {
            nodes: vec![QuadTreeNode::new_leaf(map_bounds, shard)],
            root: 0,
        }
    }

    pub fn split_and_fuse(&mut self, shard_manager: &mut ShardManager, entities: &[Entity]) -> (usize, bool)
    {
        let mut shard_allocation_count = 0;
        let mut has_split_or_fused = false;
        let mut node_to_visit = vec![self.root];
        
        while let Some(current_node) = node_to_visit.pop()
        {
            match self.nodes[current_node].node_type
            {
                QuadTreeNodeType::Node(children) =>
                {
                    let mut is_all_leaf = true;
                    for child in children
                    {
                        if let QuadTreeNodeType::Node(_) = self.nodes[child].node_type
                        {
                            is_all_leaf = false;
                        }

                        node_to_visit.push(child);
                    }

                    if is_all_leaf
                    {
                        let mut missing_shard = false;

                        let shards_load = children.into_iter()
                            .map(|child|
                            {
                                let entity_count = match self.get_entity_count(shard_manager, child)
                                {
                                    Some(entity_count) => entity_count,
                                    None =>
                                    {
                                        missing_shard = true;
                                        return None;
                                    }
                                };

                                Some(entity_count)
                            })
                            .fold(0_usize, |acc, entities_count| {
                                match entities_count
                                { 
                                    Some(entities_count) => acc + entities_count,
                                    None => acc,
                                }
                            });

                        if missing_shard
                        {
                            continue;
                        }

                        if shards_load >= PLAYER_LIMIT
                        {
                            continue;
                        }
                        
                        self.fuse(current_node, shard_manager, shards_load);
                        has_split_or_fused = true;
                    }
                }
                QuadTreeNodeType::Leaf(_) =>
                {
                    let entity_count = match self.get_entity_count(shard_manager, current_node)
                    {
                        Some(entity_count) => entity_count,
                        None => continue,
                    };

                    if entity_count > PLAYER_LIMIT
                    {

                        println!("free_shards count: {:?}", shard_manager.free_shards);
                        println!("shards count: {}", shard_manager.shards.len());
                        match self.split_leaf(current_node, shard_manager, entities) {
                            None => {
                                //println!("split_leaf failed");
                                shard_allocation_count += 3;
                            }
                            Some(_) => {
                                println!("split_leaf successful");
                                has_split_or_fused = true;
                            }
                        }
                    }
                }
            }
        }

        (shard_allocation_count, has_split_or_fused)
    }
    
    fn get_entity_count(&mut self, shard_manager: &mut ShardManager, leaf: usize) -> Option<usize>
    {
        match self.nodes[leaf].node_type
        {
            QuadTreeNodeType::Node(_) => None,
            QuadTreeNodeType::Leaf(ref mut shard_id) =>
            {
                match shard_manager.get_entity_count(*shard_id)
                {
                    Ok(count) => Some(count),
                    Err(error) =>
                    {
                        match error
                        {
                            ShardManagerError::ShardNotFound => None,
                            ShardManagerError::ShardReplaced(new_id) =>
                            {
                                *shard_id = new_id;
                                
                                let entity_count = shard_manager.get_entity_count(new_id)
                                    .expect("If the resolved id was not present, we would have gotten ShardNotFound");
                                Some(entity_count)
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn new_leaf(&mut self, bounds: Rect, shard: ShardId) -> usize
    {
        self.nodes.push(QuadTreeNode::new_leaf(bounds, shard));
        self.nodes.len() - 1
    }
    
    fn leaf_to_node(&mut self, leaf: usize, quadrants: [usize; 4])
    {
        self.nodes[leaf].node_type = QuadTreeNodeType::Node(quadrants);
    }
    
    pub fn split_leaf(&mut self, leaf: usize, shard_manager: &mut ShardManager, entities: &[Entity]) -> Option<()>
    {
        let quadrants = self.nodes[leaf].bounds.divide();
        let counts = Self::split_entity_count(quadrants, entities);
     
        let old_shard_id = match self.nodes[leaf].node_type
        {
            QuadTreeNodeType::Leaf(shard) => shard,
            QuadTreeNodeType::Node(_) => return None,
        };

        if shard_manager.free_shards.iter().count() < 3 {
            println!("Pas assez de shards libres: {:?}", shard_manager.free_shards.iter().count());
            return None;
        }

        shard_manager.release_shard(old_shard_id);
        
        let quadrants =
        [
            self.new_leaf(quadrants[0],
                          shard_manager.new_shard_with_capacity(quadrants[0], quadrants[0], counts[0])?),
            self.new_leaf(quadrants[1],
                          shard_manager.new_shard_with_capacity(quadrants[1], quadrants[1], counts[1])?),
            self.new_leaf(quadrants[2],
                          shard_manager.new_shard_with_capacity(quadrants[2], quadrants[2], counts[2])?),
            self.new_leaf(quadrants[3],
                          shard_manager.new_shard_with_capacity(quadrants[3], quadrants[3], counts[3])?),
        ];
        
        self.leaf_to_node(leaf, quadrants);
        Some(())
    }

    fn split_entity_count(quadrants: [Rect; 4], entities: &[Entity]) -> [usize; 4]
    {
        [
            entities.iter()
                .filter(|entity| { entity.position().overlap_rect(quadrants[0]) })
                .count(),
            entities.iter()
                .filter(|entity| { entity.position().overlap_rect(quadrants[1]) })
                .count(),
            entities.iter()
                .filter(|entity| { entity.position().overlap_rect(quadrants[2]) })
                .count(),
            entities.iter()
                .filter(|entity| { entity.position().overlap_rect(quadrants[3]) })
                .count(),
        ]
    }
    
    pub fn fuse(&mut self, parent: usize, shard_manager: &mut ShardManager, shards_load: usize) -> Option<()>
    {
        let mut children = match self.nodes[parent].node_type
        {
            QuadTreeNodeType::Node(children) => children.to_vec(),
            QuadTreeNodeType::Leaf(_) => return None,
        };
        
        let node = self.nodes.get_mut(children.pop().unwrap()).unwrap();
        let shard_id = match node.node_type
        {
            QuadTreeNodeType::Node(_) => unreachable!(),
            QuadTreeNodeType::Leaf(shard) => shard,
        };
        shard_manager.update_shard(shard_id,node.bounds, shards_load);
        
        for child in children
        {
            let shard_id = match self.nodes[child].node_type
            {
                QuadTreeNodeType::Node(_) => unreachable!(),
                QuadTreeNodeType::Leaf(shard) => shard,
            };
            
            shard_manager.release_shard(shard_id);
        }
        
        Some(())
    }

    pub fn leaf_for(&self, position: Position) -> Option<usize>
    {
        if !position.overlap_rect(self.nodes[self.root].bounds)
        {
            return None
        }

        let mut current_node = self.root;
        loop
        {
            match self.nodes[current_node].node_type
            {
                QuadTreeNodeType::Node(children) =>
                {
                    for child in children.into_iter()
                    {
                        if position.overlap_rect(self.nodes[child].bounds)
                        {
                            current_node = child;
                            continue;
                        }
                    }
                }
                QuadTreeNodeType::Leaf(_) =>
                {
                    return Some(current_node);
                }
            }
        }
    }

    pub fn shard_for(&self, position: Position) -> Option<ShardId>
    {
        let leaf = self.leaf_for(position)?;
        
        match self.nodes[leaf].node_type
        {
            QuadTreeNodeType::Node(_) => None,
            QuadTreeNodeType::Leaf(shard) => Some(shard)
        }
    }

    pub fn shards_near(&self, shard_manager: &mut ShardManager, circle: Circle) -> Vec<ShardId>
    {
        let mut shards = Vec::new();

        let mut node_to_visit = vec![self.root];
        loop
        {
            let current_node = match node_to_visit.pop()
            {
                Some(node) => node,
                None => return shards,
            };

            if !circle.center.overlap_rect(self.nodes[current_node].bounds)
            {
                continue;
            }

            match self.nodes[current_node].node_type
            {
                QuadTreeNodeType::Node(children) =>
                {
                    for child in children.into_iter()
                    {
                        node_to_visit.push(child);
                    }
                }
                QuadTreeNodeType::Leaf(shard_id) =>
                {
                    if shard_manager.should_resolve(shard_id)
                    {
                        let resolved = match shard_manager.resolve_id(shard_id)
                        {
                            Some(resolved) => resolved,
                            None => continue,
                        };
                        shards.push(resolved);
                    }
                    else
                    {
                        shards.push(shard_id);
                    }
                }
            }
        }
    }
}
