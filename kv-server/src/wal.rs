#![allow(unused)]

use crate::DbPaths;
use crc32c::crc32c;
use protocol::BinCodec;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::os::fd;
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
    pub fragment: FragmentType,
    pub payload_length: u16,
    pub checksum: u32,
}


impl FragmentHeader {
    fn new(fragment: FragmentType, payload_length: u16, payload: &[u8]) -> Self {
        let checksum = generate_checksum(fragment.clone(), payload).unwrap();
        Self {
            fragment,
            payload_length,
            checksum,
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

impl BinCodec for Operation{}

#[derive(Serialize, Deserialize)]
pub enum Operation {
    Put { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
    Create { column: String },
    Append,
    Write,
    SingleDelete,
}


pub struct LogWriter {
    log_file: LogFile,
    block_offset: usize
}

pub struct LogReader {
    log_file: LogFile,
    block_offset: usize
}


pub struct LogFile{
    file: File,
}


impl LogFile {
    fn new(path: PathBuf) -> Self { todo!()}
    fn append(&mut self, length: usize, data: &[u8]) {
        
    }
    fn close(&self) {}

    fn flush(&self){}

    fn sync(&self){}
}


impl LogWriter {
    fn add_record(&mut self, record: &[u8]) -> Result<(), String > { 
        let mut ptr = 0; 
        let mut left = record.len();
        
        let mut begin = true;

        while left > 0  {
            let leftover = BLOCK_SIZE - self.block_offset; 

            if leftover < HEADER_SIZE {
                if (leftover > 0) {
                    let padding = vec![0u8; leftover];
                    self.log_file.append(leftover, &padding);
                }

                self.block_offset = 0
            }

            let avail = BLOCK_SIZE - self.block_offset - HEADER_SIZE;

            let fragment_length = if left < avail {
                left 
            } else {
                avail
            };

            let end = left == fragment_length;

            let mut fragment_type: FragmentType; 

            if begin && end {
                fragment_type  = FragmentType::Full;
            } else if begin {
                fragment_type = FragmentType::First
            } else if end {
                fragment_type = FragmentType::Last
            } else {
                fragment_type = FragmentType::Middle;
            }

            self.emit_record(fragment_type, &record[ptr..fragment_length], fragment_length as u16);

            left -=  fragment_length;
            ptr += fragment_length;
            begin = false;
        }
        
        todo!()
    }

    fn emit_record(&mut self, fragment_type: FragmentType, record: &[u8],  length: u16) {
        let fragment = FragmentHeader::new(fragment_type, length, record).serialize_record().unwrap();

        self.log_file.append(fragment.len(), &fragment);
        self.log_file.append(length as usize, record);
        self.log_file.flush();
        self.block_offset += HEADER_SIZE + length as usize;
    }

}

pub struct WalManager {
    blocks: u32,
    writer: LogWriter
}


pub struct BlockHeader {

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

