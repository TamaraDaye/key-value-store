#![allow(unused)]

use std::ops::Range;
use rand;

pub struct Arena {}

pub struct MemTable {
    nodes: Vec<Node>,
    bytes: Vec<u8>,
}

type NodeId = u32;

pub struct BTree<T> {
    root: BTreeNode<T>,
}

pub struct BTreeNode<T> {
    keys: Vec<T>,
    is_leaf: bool,
}

pub struct Node {
    key: Vec<u8>,
    value: Vec<u8>,
    forward: Vec<Option<NodeId>>,
}

pub struct SkipList {
    max_height: u32,
    p: f32,
    head: Node,
    height: u32,
    size: u32,
}

impl SkipList {
    fn new(levels: u32, p: f32) -> Self {
        let mut head: Node = Node {
            key: vec![],
            value: vec![],
            forward: vec![None; levels as usize],
        };
        Self {
            max_height: levels,
            p,
            head,
            height: 0,
            size: 0,
        }
    }


    fn random_height(&self) -> u32 {
        let mut h = 1;

        while h < self.max_height && rand::random::<f32>() < self.p {
            h += 1;
        }

        return h
    }


    fn predecessors(&mut self, key: Vec<u8>) -> bool {
        let levels = self.max_height;
        let mut update = [self.head; levels];
        true 
    }
    
}
