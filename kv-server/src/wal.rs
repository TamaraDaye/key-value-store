#![allow(unused)]

use crate::DbPaths;
use crc32c::crc32c;
use protocol::BinCodec;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;

const BLOCK_SIZE: usize = 32 <<  10;
const SEGMENT_SIZE: usize = 16 << 20;
const HEADER_SIZE: usize = 7; 
const MAX_RECORD_SIZE: usize = 8 << 20;

pub fn generate_checksum(fragment_type: FragmentType, payload: &[u8]) -> Result<u32, String> {
    let length = u16::try_from(payload.len()).unwrap();

    let mut input = Vec::with_capacity(3 + payload.len());
    input.push(fragment_type as u8);
    input.extend_from_slice(&length.to_le_bytes());
    input.extend_from_slice(payload);

    Ok(crc32c(&input))
}

#[derive(Serialize, Deserialize)]
pub struct FragmentHeader {
    checksum: u32,
    fragment: FragmentType,
    payload_length: u32,
}

impl FragmentHeader {
    fn new(fragment: FragmentType, payload_length: u32, payload: &[u8]) -> Self {
        let checksum = generate_checksum(fragment.clone(), payload).unwrap();
        Self {
            checksum,
            fragment,
            payload_length,
        }
    }
}

impl BinCodec for FragmentHeader {}

#[repr(u8)]
#[derive(Serialize, Deserialize, Clone)]
enum FragmentType {
    Full = 1,
    First = 2,
    Middle = 3,
    Last = 4,
}

impl BinCodec for FragmentType {}

impl WalManager {
    fn write(&mut self, record: WalRecord) -> Result<(), String > { 
        let data = record.serialize_record().unwrap();
        if data.len() > BLOCK_SIZE {

        }
        let fragment_type = FragmentType::
        Ok(())
    }


    fn write_fragments(&mut self, data: &[u8]) -> Result<(), String> {
        let mut offset = 0;
        let mut first = true; 


        while offset < data.len() {
            let remaining = BLOCK_SIZE - self.active_block;

            if remaining <= 
        }
    }
    fn read() {}
}

#[derive(Serialize, Deserialize)]
pub enum Operation {
    Put { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
    Create { column: String },
    Append,
    Write,
    SingleDelete,
}
pub struct Wal {
    size: usize,
    enabled: bool,
    version: usize,
    manager: Option<WalManager>,
}

pub struct WalManager {
    file: File,
    blocks: u32,
    active_block: u32,
    block_size: usize,
}

impl WalManager {
    fn new(file: File) -> Self {
        Self {
            file,
            blocks: 0,
            active_block: 0,
            block_size: BLOCK_SIZE,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct WalRecord {
    sequence_number: u64,
    operation: Operation,
}

impl BinCodec for WalRecord {}

impl WalRecord {
    fn new(sequence_number: u64, operation: Operation) -> Self {
        Self {
            sequence_number, 
            operation,
        }
    }
}

impl Wal {
    fn new(version: usize) -> Self {
        Self {
            size: 0,
            enabled: false,
            version: version,
            manager: None,
        }
    }

    fn disable_wal(&mut self) {
        self.enabled = false
    }
}
