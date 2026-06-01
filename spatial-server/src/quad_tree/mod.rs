use std::marker::PhantomData;
use crate::geometry::prelude::*;
use crate::network_object::shard::{Shard, ShardId, ShardManager};

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


const PLAYER_LIMIT: usize = 100;

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

    pub fn split_and_fuse(&mut self, shard_manager: &mut ShardManager)
    {
        let mut node_to_visit = vec![self.root];
        loop
        {
            let current_node = match node_to_visit.pop()
            {
                Some(node) => node,
                None => return,
            };

            match self.nodes[current_node].node_type
            {
                QuadTreeNodeType::Node(children) =>
                {
                    let mut is_all_children = true;
                    for child in children
                    {
                        if let QuadTreeNodeType::Node(_) = self.nodes[child].node_type
                        {
                            is_all_children = false;
                        }

                        node_to_visit.push(child);
                    }

                    if is_all_children
                    {
                        let mut missing_shard = false;

                        let shards_load = children.iter()
                            .map(|child| {
                                let shard_id = match self.nodes[*child].node_type
                                {
                                    QuadTreeNodeType::Node(_) => unreachable!(),
                                    QuadTreeNodeType::Leaf(shard) => shard,
                                };

                                let shard = match shard_manager.get_shard(shard_id)
                                {
                                    Some(shard) => shard,
                                    None =>
                                        {
                                            missing_shard = true;
                                            return None;
                                        },
                                };

                                Some(shard.entities_count)
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
                        
                        self.fuse(current_node, shard_manager);
                        // TODO : implement fuse method, putting all but one shard back into the free list
                    }
                }
                QuadTreeNodeType::Leaf(shard_id) =>
                {
                    let shard = match shard_manager.get_shard(shard_id)
                    {
                        Some(shard) => shard,
                        None => continue,
                    };

                    if shard.entities_count < PLAYER_LIMIT
                    {
                        self.split_leaf(current_node, shard_manager);
                        // TODO : refactor split leaf to split if there are shards in the free list, otherwise asks for more shards
                    }
                }
            }
        }
    }
    
    // pub fn tmp(shard_generator: &mut ShardGenerator, map_bounds: Rect, entities: &[Position]) -> Self
    // {
    //     if entities.len() <= PLAYER_LIMIT
    //     {
    //         return Self
    //         {
    //             bounds: map_bounds,
    //             inner: QuadTreeInner::Leaf(shard_generator.get_shard()),
    //         };
    //     }
    // 
    //     let entites_quadrant = |bound: Rect|
    //         {
    //             entities.iter().copied()
    //                 .filter(|entity| entity.overlap_rect(bound))
    //                 .collect::<Vec<_>>()
    //         };
    // 
    //     let quadrant_bounds = map_bounds.divide();
    // 
    //     let entities_0 = entites_quadrant(quadrant_bounds[0]);
    //     let entities_1 = entites_quadrant(quadrant_bounds[0]);
    //     let entities_2 = entites_quadrant(quadrant_bounds[0]);
    //     let entities_3 = entites_quadrant(quadrant_bounds[0]);
    // 
    // 
    //     Self
    //     {
    //         bounds: map_bounds,
    //         inner: QuadTreeInner::Node([
    //             Box::new(Self::new(shard_generator, quadrant_bounds[0], entities_0.as_ref())),
    //             Box::new(Self::new(shard_generator, quadrant_bounds[1], entities_1.as_ref())),
    //             Box::new(Self::new(shard_generator, quadrant_bounds[2], entities_2.as_ref())),
    //             Box::new(Self::new(shard_generator, quadrant_bounds[3], entities_3.as_ref())),
    //         ]),
    //     }
    // }
    
    fn new_leaf(&mut self, bounds: Rect, shard: ShardId) -> usize
    {
        self.nodes.push(QuadTreeNode::new_leaf(bounds, shard));
        self.nodes.len() - 1
    }
    
    fn into_node(&mut self, leaf: usize, quadrants: [usize; 4])
    {
        self.nodes[leaf].node_type = QuadTreeNodeType::Node(quadrants);
    }
    
    pub fn split_leaf(&mut self, leaf: usize)
    {
        let quadrants = self.nodes[leaf].bounds.divide();
        let quadrants =
        [
            self.new_leaf(quadrants[0]),
            self.new_leaf(quadrants[1]),
            self.new_leaf(quadrants[2]),
            self.new_leaf(quadrants[3]),
        ];
        
        self.into_node(leaf, quadrants)
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
            QuadTreeNodeType::Node(children) => None,
            QuadTreeNodeType::Leaf(shard) => Some(shard)
        }
    }

    pub fn shards_near(&self, circle: Circle) -> Vec<ShardId>
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
                    shards.push(shard_id);
                }
            }
        }
    }
}
