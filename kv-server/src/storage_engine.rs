#![allow(unused)]
use std::cmp::Ordering;
use std::collections::{VecDeque, BinaryHeap};
use std::sync::Arc;

const MAX_HEIGHT: usize = 16;
const NULL_IDX: u32 = u32::MAX;
const MEMTABLE_SIZE: usize = 16 << 20;

#[derive(Clone, Copy, Debug, Default)]
pub struct ByteSlice {
    pub offset: u32,
    pub len: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum ValueEntry {
    Value(ByteSlice),
    Tombstone,
}

pub struct SkipNode {
    pub key: ByteSlice,
    pub value: ValueEntry,
    pub height: u8,
    pub next: [u32; MAX_HEIGHT],
    pub seq_number: u32,
}

pub struct MemTable {
    byte_arena: Vec<u8>,
    nodes: Vec<SkipNode>,
    current_max_height: usize,
    rng_state: u64,
    active_memtable, 
    frozen_memtables: Vec<Arc<MemTable>>
}

impl MemTable {
    pub fn new(capacity_bytes: usize) -> Self {
        let head = SkipNode {
            key: ByteSlice::default(),
            value: ValueEntry::Tombstone,
            height: MAX_HEIGHT as u8,
            next: [NULL_IDX; MAX_HEIGHT],
            seq_number: 0,
        };

        let mut nodes = Vec::with_capacity(4096);

        nodes.push(head);

        Self {
            byte_arena: Vec::with_capacity(capacity_bytes),
            nodes,
            current_max_height: 1,
            rng_state: 0x853c49e6748fea9b,
        }
    }

    pub fn approximate_size(&self) -> usize {
        self.byte_arena.len() + (self.nodes.len() * std::mem::size_of::<SkipNode>())
    }

    pub fn alloc_bytes(&mut self, data: &[u8]) -> ByteSlice {
        let offset = self.byte_arena.len() as u32;
        let len = data.len() as u32;
        self.byte_arena.extend_from_slice(data);
        ByteSlice { offset, len }
    }

    #[inline]
    fn get_bytes(&self, slice: ByteSlice) -> &[u8] {
        let start = slice.offset as usize;
        let end = start + (slice.len as usize);
        &self.byte_arena[start..end]
    }

    fn random_height(&mut self) -> usize {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;

        self.rng_state = x;

        let height = 1 + (x.trailing_ones() as usize);
        height.min(MAX_HEIGHT)
    }

    fn find_predecessors(&self, target_key: &[u8]) -> ([u32; MAX_HEIGHT], Option<u32>) {
        let mut update = [0u32; MAX_HEIGHT];
        let mut curr = 0u32;

        for level in (0..self.current_max_height).rev() {
            loop {
                let next = self.nodes[curr as usize].next[level];
                if next == NULL_IDX {
                    break;
                };
                let next_key = self.get_bytes(self.nodes[next as usize].key);

                match next_key.cmp(target_key) {
                    Ordering::Less => curr = next,
                    Ordering::Equal => {
                        update[level] = curr;
                        if level == 0 {
                            return (update, Some(next));
                        }
                        break;
                    }
                    Ordering::Greater => break,
                }
            }
            update[level] = curr;
        }

        let candidate = self.nodes[curr as usize].next[0];

        let exact_match = if candidate != NULL_IDX
            && self.get_bytes(self.nodes[candidate as usize].key) == target_key
        {
            Some(candidate)
        } else {
            None
        };

        (update, exact_match)
    }

    pub fn put(&mut self, key: &[u8], value: &[u8]) {
        let val_slice = self.alloc_bytes(value);
        self.insert_entry(key, ValueEntry::Value(val_slice));
    }

    pub fn delete(&mut self, key: &[u8]) {
        todo!()
    }

    fn insert_entry(&mut self, key: &[u8], value_entry: ValueEntry) {
        let (mut update, exact_match) = self.find_predecessors(key);


        if let Some(existing_idx) = exact_match {
            self.nodes[existing_idx as usize].value = value_entry;
            return;
        }

        let height = self.random_height();

        if height > self.current_max_height {
            for level in self.current_max_height..height {
                update[level] = 0
            }
            self.current_max_height = height;
        };

        let key_slice = self.alloc_bytes(key);
        let new_node_idx = self.nodes.len() as u32;

        let mut new_node = SkipNode {
            key: key_slice, 
            value: value_entry, 
            height: height as u8, 
            seq_number: 0, 
            next: [NULL_IDX; MAX_HEIGHT], 
        };


        for level in 0..height {
            let prev_idx = update[level] as usize; 
            new_node.next[level] = self.nodes[prev_idx].next[level];
            self.nodes[prev_idx].next[level] = new_node_idx;
        }

        self.nodes.push(new_node);
    }
}
