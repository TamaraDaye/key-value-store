#![allow(unused)]
#![feature(write_all_vectored)]

use crate::storage_engine::{OpType, TableEntry, Writer};
use std::fs::File;
use std::io::{self, ErrorKind, IoSlice, Write};

pub const ENTRY_HEADER_SIZE: usize = 13;
pub const IO_BUFFER_CAPACITY: usize = 1024;
pub const DEFAULT_BLOCK_SIZE: usize = 32 * 1024;
pub const FOOTER_SIZE: usize = 24;
pub const TABLE_MAGIC_NUMBER: u64 = 0x5253_4944_4955_4D31;

#[derive(Default, Debug)]
pub struct TableMetaData {
    pub data_size: u64,
    pub index_size: u64,
    pub raw_key_size: u64,
    pub number_of_entries: u64,
    pub data_blocks: u64,
}

#[derive(Debug)]
pub struct BlockIndex {
    pub max_key: Vec<u8>,
    pub max_seq: u64,
    pub block_offset: u64,
    pub block_size: u32,
}

#[derive(Debug)]
pub struct TableFooter {
    pub index_block_offset: u64,
    pub index_block_len: u64,
    pub magic_number: u64,
}

impl TableFooter {
    pub fn encode(&self) -> [u8; FOOTER_SIZE] {
        let mut bytes = [0u8; FOOTER_SIZE];

        bytes[0..8].copy_from_slice(&self.index_block_offset.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.index_block_len.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.magic_number.to_le_bytes());

        bytes
    }
}

pub struct BlockOffsetDirectory {
    pub num_entries: u32,
    bytes: Vec<u8>,
}

impl BlockOffsetDirectory {
    pub fn with_capacity(max_entries: usize) -> io::Result<Self> {
        let capacity = max_entries
            .checked_mul(4)
            .and_then(|n| n.checked_add(4))
            .ok_or_else(|| {
                io::Error::new(ErrorKind::InvalidInput, "directory capacity overflow")
            })?;

        Ok(Self {
            num_entries: 0,
            bytes: Vec::with_capacity(capacity),
        })
    }

    pub fn clear(&mut self) {
        self.num_entries = 0;
        self.bytes.clear();
    }

    pub fn push(&mut self, offset: u32) -> io::Result<()> {
        self.bytes.extend_from_slice(&offset.to_le_bytes());

        self.num_entries = self
            .num_entries
            .checked_add(1)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "too many block entries"))?;

        Ok(())
    }

    pub fn finish(&mut self) {
        self.bytes
            .extend_from_slice(&self.num_entries.to_le_bytes());
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }
}

fn encode_entry_header(
    dst: &mut [u8; ENTRY_HEADER_SIZE],
    key_len: usize,
    seq_num: u64,
    op_type: OpType,
) -> io::Result<()> {
    let key_len = u32::try_from(key_len)
        .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "key length exceeds u32"))?;

    dst[0..4].copy_from_slice(&key_len.to_le_bytes());
    dst[4..12].copy_from_slice(&seq_num.to_le_bytes());
    dst[12] = op_type as u8;

    Ok(())
}

pub struct SSTableBuilder<W: Writer> {
    writer: W,

    sparse_index: Vec<BlockIndex>,
    metadata: TableMetaData,

    file_offset: u64,

    max_block_bytes: usize,
    iov_max: usize,

    magic_number: u64,
    failed: bool,
}

impl<W: Writer> SSTableBuilder<W> {
    pub fn new(writer: W, max_block_bytes: usize, iov_max: usize) -> io::Result<Self> {
        Self::with_magic(writer, max_block_bytes, iov_max, TABLE_MAGIC_NUMBER)
    }

    pub fn with_magic(
        writer: W,
        max_block_bytes: usize,
        iov_max: usize,
        magic_number: u64,
    ) -> io::Result<Self> {
        if iov_max < 4 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "iovec capacity must be at least 4",
            ));
        }

        if max_block_bytes > u32::MAX as usize {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "block size exceeds u32 offset range",
            ));
        }

        Ok(Self {
            writer,
            sparse_index: Vec::new(),
            metadata: TableMetaData::default(),
            file_offset: 0,
            max_block_bytes,
            iov_max,
            magic_number,
            failed: false,
        })
    }
}

