use std::collections::HashMap;
use bytes::{Bytes, BytesMut};
use crate::{Deserializable, Serializable, SerializationError};

#[derive(Clone, Debug, PartialEq)]
pub struct TopicTree {
    pub name: String,
    pub item: TopicTreeType
}

impl TopicTree {
    pub fn new(name: String, item: TopicTreeType) -> Self {
        Self { name, item }
    }

    pub fn new_empty(name: String) -> Self {
        let item = TopicTreeType::Node(TopicNode::new(Vec::<TopicTree>::new()));
        Self { name, item }
    }

    pub fn add_leaf(&mut self,name: String, data: Vec<u8>) {
        match &mut self.item {
            TopicTreeType::Leaf(_) => panic!("Cannot add leaf to leaf node"),
            TopicTreeType::Node(nodes) => {
                nodes.add_leaf(name,TopicLeaf::new(data))
            }
        }
    }

    pub fn add_tree(&mut self,tree: TopicTree) {
        match &mut self.item {
            TopicTreeType::Leaf(_) => panic!("Cannot add leaf to leaf node"),
            TopicTreeType::Node(nodes) => {
                nodes.add_tree(tree)
            }
        }
    }

    pub fn get_child(&self, name: &str) -> Option<&TopicTree> {
        match &self.item {
            TopicTreeType::Node(node) => node.data.iter().find(|t| t.name == name),
            TopicTreeType::Leaf(_) => None,
        }
    }

    pub fn get_child_owned(self, name: &str) -> Option<TopicTree> {
        match self.item {
            TopicTreeType::Node(node) => node.data.into_iter().find(|t| t.name == name),
            TopicTreeType::Leaf(_) => None,
        }
    }

    pub fn keys(self) -> Vec<Vec<u8>> {
        match self.item {
            TopicTreeType::Leaf(_) => vec!(Vec::from(self.name)),
            TopicTreeType::Node(nodes) =>
                {
                let trees = nodes.data;
                let mut v = Vec::<Vec<u8>>::new();
                for tree in trees {
                    let keys = tree.keys();
                    for mut key in keys {
                        key.splice(0..0,Vec::from(self.name.clone()+"/"));
                        v.push(key);
                    }
                }
                v
            }
        }
    }

    pub fn flatten(self) -> HashMap<Vec<u8>, Vec<u8>> {
        match self.item {
            TopicTreeType::Leaf(topic) => {
                let mut map = HashMap::<Vec<u8>,Vec<u8>>::new();
                map.insert(Vec::from(self.name), topic.data);
                map
            },
            TopicTreeType::Node(nodes) => {
                let mut map = HashMap::<Vec<u8>,Vec<u8>>::new();
                for topic in nodes.data {
                    let map_ = topic.flatten();
                    for (mut key, value) in map_ {
                        key.splice(0..0,Vec::from(self.name.clone()+"/"));
                        map.insert(key,value);
                    }
                }
                map
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        let parts: Vec<&str> = key.splitn(2, '/').collect();
        let first = parts[0];
        let rest = parts.get(1).copied();

        if self.name != first {
            return None;
        }

        match (&self.item, rest) {
            (TopicTreeType::Leaf(leaf), None) => Some(&leaf.data),
            (TopicTreeType::Node(nodes), Some(remaining)) => {
                for child in &nodes.data {
                    if let Some(data) = child.get(remaining) {
                        return Some(data);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn get_sub_tree(&self, key: &str) -> Option<TopicTree> {
        match key {
            "*" => Some(self.clone()),
            &_ => {
                let parts: Vec<&str> = key.splitn(2, '/').collect();
                let first = parts[0];
                let rest = parts.get(1).copied();

                if self.name != first {
                    return None;
                }

                match (&self.item, rest) {
                    (_, None) => Some(self.clone()),
                    (TopicTreeType::Node(nodes), Some(remaining)) => {
                        let mut sub_tree = Self::new_empty(self.clone().name);
                        for child in &nodes.data {
                            if let Some(data) = child.get_sub_tree(remaining) {
                                sub_tree.add_tree(data)
                            }
                        }
                        let TopicTreeType::Node(topic_node) = sub_tree.clone().item else { return None };
                        match topic_node.data.iter().count() {
                            0 => None,
                            _ => Some(sub_tree),
                        }
                    }
                    _ => None,
                }
            },
        }
    }

    pub fn merge(&mut self, other: &TopicTree) {
        if self.name != other.clone().name { return }

        match &mut self.item {
            TopicTreeType::Leaf(_) => return,
            TopicTreeType::Node(self_topic) => {
                let other_children = match &other.item {
                    TopicTreeType::Node(n) => &n.data,
                    TopicTreeType::Leaf(_) => return,
                };

                for other_tree in other_children {
                    let has_similar_entry = self_topic.data.iter_mut()
                        .find(|t| t.name == other_tree.name);

                    match has_similar_entry {
                        Some(self_tree) => self_tree.merge(other_tree),
                        None => self_topic.add_tree(other_tree.clone()),
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TopicTreeType {
    Leaf(TopicLeaf),
    Node(TopicNode),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopicNode{
    pub data: Vec<TopicTree>
}

impl TopicNode {
    pub fn new(data: Vec<TopicTree>) -> Self {
        Self { data }
    }
    pub fn add_leaf(&mut self,name: String, leaf: TopicLeaf){

        self.add_tree(TopicTree::new(name,TopicTreeType::Leaf(leaf)));
    }

    pub fn add_tree(&mut self, tree: TopicTree){
        self.data.push(tree);
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct TopicLeaf {
    pub data: Vec<u8>
}

impl TopicLeaf {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn data(self) -> Vec<u8> { self.data }
}

#[repr(u8)]
pub enum TopicTreeTag {
    Node = 0x01,
    Leaf = 0x02,
}

impl TryFrom<u8> for TopicTreeTag {
    type Error = SerializationError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(TopicTreeTag::Node),
            0x02 => Ok(TopicTreeTag::Leaf),
            _ => Err(SerializationError::InvalidDeserializationState)
        }
    }
}

impl Serializable for TopicTree {
    fn serialize(self, buffer: &mut BytesMut) -> Result<(), SerializationError> {
        self.name.serialize(buffer)?;
        self.item.serialize(buffer)?;
        Ok(())
    }
}

impl Deserializable for TopicTree {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let name = String::deserialize(bytes)?;
        let item = TopicTreeType::deserialize(bytes)?;
        Ok(Self { name, item })
    }
}

impl Serializable for TopicTreeType {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        match self {
            TopicTreeType::Node(nodes) => {
                (TopicTreeTag::Node as u8).serialize(bytes)?;
                nodes.serialize(bytes)?
            },
            TopicTreeType::Leaf(data) => {
                (TopicTreeTag::Leaf as u8).serialize(bytes)?;
                data.serialize(bytes)?
            }
        }
        Ok(())
    }
}

impl Deserializable for TopicTreeType {
    fn deserialize(mut bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let tag = u8::deserialize(&mut bytes)?;
        let packet_tag = TopicTreeTag::try_from(tag)?;
        let data = match packet_tag {
            TopicTreeTag::Node => Self::Node(TopicNode::deserialize(&mut bytes)?),
            TopicTreeTag::Leaf => Self::Leaf(TopicLeaf::deserialize(&mut bytes)?),
        };
        Ok(data)
    }
}

impl Serializable for TopicNode {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.data.serialize(bytes)?;
        Ok(())
    }
}

impl Deserializable for TopicNode {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let data = Vec::<TopicTree>::deserialize(bytes)?;
        Ok(Self { data })
    }
}

impl Serializable for TopicLeaf {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.data.serialize(bytes)?;
        Ok(())
    }
}

impl Deserializable for TopicLeaf {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let data = Vec::<u8>::deserialize(bytes)?;
        Ok(Self { data })
    }
}