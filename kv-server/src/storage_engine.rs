#![allow(unused)]
use crate::sst::SSTable;
use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;


const MAX_HEIGHT: usize = 16;
const NULL_IDX: u32 = u32::MAX;
const MEMTABLE_SIZE: usize = 16 << 20;
const RNG_STATE: usize = 0x853c49e6748fea9b;

pub trait Writer: std::io::Write {
    fn open(data_dir: Option<&Path>);
    fn sync_data(&mut self);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ByteSlice {
    pub offset: u32,
    pub len: u32,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum ValueEntry {
    Value(ByteSlice),
    #[default]
    Tombstone,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpType {
    Put = 1,
    Delete = 2,
}

impl TryFrom<u8> for OpType {
    type Error = u8;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(OpType::Put),
            2 => Ok(OpType::Delete),
            invalid => Err(invalid),
        }
    }
}

pub struct TableEntry<'a> {
    pub op_type: OpType,
    pub key_bytes: &'a [u8],
    pub seq_num: u64,
    pub value_bytes: &'a [u8],
}

pub struct Engine <W: Writer>{
    ssfile: SSTable<W>,
    active_memtable: MemTable,
    frozen_memtables: Option<VecDeque<Arc<MemTable>>>,
}
pub struct SkipNode {
    pub key: ByteSlice,
    pub value: ValueEntry,
    pub seq_number: u64,
    pub height: u8,
    pub next: [u32; MAX_HEIGHT],
}


pub struct MemTable {
    column_family: String,
    byte_arena: Vec<u8>,
    nodes: Vec<SkipNode>,
    current_max_height: usize,
    rng_state: usize,
}

pub struct MemTableIterator<'a> {
    curr_node: usize,
    mem_table: &'a MemTable,
}

impl<'a> Iterator for MemTableIterator<'a> {
    type Item = TableEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_node == NULL_IDX as usize {
            return None;
        }

        let next_idx = self.mem_table.nodes[self.curr_node].next[0] as usize;

        self.curr_node = next_idx;

        if next_idx == NULL_IDX as usize {
            return None;
        }

        let node = &self.mem_table.nodes[next_idx];
        let (op_type, value_bytes) = match node.value {
            ValueEntry::Value(slice) => (OpType::Put, self.mem_table.get_bytes(slice)),
            ValueEntry::Tombstone => (OpType::Delete, &[][..]),
        };

        let key_bytes = self.mem_table.get_bytes(node.key);

        Some(TableEntry {
            op_type,
            key_bytes,
            seq_num: node.seq_number as u64,
            value_bytes,
        })
    }
}
impl MemTable {
    pub fn new(capacity_bytes: usize, column_family: String) -> Self {
        let head = SkipNode {
            key: ByteSlice::default(),
            value: ValueEntry::Tombstone,
            seq_number: 0,
            height: MAX_HEIGHT as u8,
            next: [NULL_IDX; MAX_HEIGHT],
        };

        let mut nodes = Vec::with_capacity(4096);
        nodes.push(head);

        Self {
            byte_arena: Vec::with_capacity(capacity_bytes),
            nodes,
            current_max_height: 1,
            rng_state: RNG_STATE,
            column_family,
        }
    }

    pub fn iter(&self) -> MemTableIterator<'_> {
        MemTableIterator {
            curr_node: 0,
            mem_table: self,
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

    #[inline]
    fn compare_internal_key(
        a_key: &[u8],
        a_seq: u64,
        b_key: &[u8],
        b_seq: u64,
    ) -> Ordering {
        a_key.cmp(b_key).then_with(|| b_seq.cmp(&a_seq))
    }

    fn find_predecessors(&self, target_key: &[u8], target_seq: u64) -> [u32; MAX_HEIGHT] {
        let mut update = [0u32; MAX_HEIGHT];
        let mut curr = 0u32;

        for level in (0..self.current_max_height).rev() {
            loop {
                let next = self.nodes[curr as usize].next[level];
                if next == NULL_IDX {
                    break;
                }

                let next_node = &self.nodes[next as usize];
                let next_key = self.get_bytes(next_node.key);

                match Self::compare_internal_key(
                    next_key,
                    next_node.seq_number,
                    target_key,
                    target_seq,
                ) {
                    Ordering::Less => curr = next,
                    Ordering::Equal | Ordering::Greater => break,
                }
            }
            update[level] = curr;
        }

        update
    }

    pub fn put(&mut self, key: &[u8], value: &[u8], seq_num: u64) {
        let val_slice = self.alloc_bytes(value);
        self.insert_entry(key, seq_num, ValueEntry::Value(val_slice));
    }

    pub fn delete(&mut self, key: &[u8], seq_num: u64) {
        self.insert_entry(key, seq_num, ValueEntry::Tombstone);
    }

    fn insert_entry(&mut self, key: &[u8], seq_num: u64, value_entry: ValueEntry) {
        let mut update = self.find_predecessors(key, seq_num);
        let height = self.random_height();

        if height > self.current_max_height {
            for level in self.current_max_height..height {
                update[level] = 0;
            }
            self.current_max_height = height;
        }

        let key_slice = self.alloc_bytes(key);
        let new_node_idx = self.nodes.len() as u32;

        let mut new_node = SkipNode {
            key: key_slice,
            value: value_entry,
            seq_number: seq_num,
            height: height as u8,
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


#[cfg(test)]
mod tests {
    use super::*;
}
