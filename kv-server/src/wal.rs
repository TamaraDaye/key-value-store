#![allow(unused)]

use crate::DbPaths;
use crc32c;
use protocol::BinCodec;
use serde::{Deserialize, Serialize, Deserializer, Serializer, de::Error};
use std::fmt::Write;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::os::fd;
use std::path::PathBuf;

const BLOCK_SIZE: usize = 32 <<  10;
const SEGMENT_SIZE: usize = 16 << 20;
const HEADER_SIZE: usize = 7; 
const MAX_RECORD_SIZE: usize = 8 << 20;

pub fn generate_checksum(fragment_type: FragmentType, record: &[u8]) -> Result<u32, String> {
    let length = u16::try_from(record.len())
        .map_err(|_| format!("record length {} exceeds u16::MAX", record.len()))?;

    let [len_lo, len_hi] = length.to_le_bytes();

    let header = [fragment_type as u8, len_lo, len_hi];

    let crc = crc32c::crc32c(&header);

    let crc = crc32c::crc32c_append(crc, record);

    Ok(crc)
}

pub struct FragmentHeader {
    pub fragment: FragmentType,
    pub record_length: u16,
    pub checksum: u32,
}


impl FragmentHeader {
    fn new(fragment: FragmentType, record_length: u16, record: &[u8]) -> [u8; 7] {
        let checksum = generate_checksum(fragment, record).unwrap();
        let mut header = [0u8; 7];
        header[0] =  fragment as u8;
        header[1..3].copy_from_slice(&record_length.to_le_bytes());
        header[3..].copy_from_slice(&checksum.to_le_bytes());

        header
    }
}


#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FragmentType {
    Full = 1,
    First = 2,
    Middle = 3,
    Last = 4,
}


impl TryFrom<u8> for FragmentType {
    type Error = u8;


    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match  value  {
            1 => Ok(FragmentType::Full),  
            2 => Ok(FragmentType::First),  
            3 => Ok(FragmentType::Middle),  
            4 => Ok(FragmentType::Last),  
            invalid => Err(invalid)

        }
    }
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
    log_file: BufWriter<File>,
    block_offset: usize
}

pub struct LogReader {
    log_file: BufReader<File>,
    block_offset: usize,
    last_record_offset: usize
}


impl LogReader {
    fn read_record(&mut self) -> WalRecord {
        let mut header: [u8; 7] = [0; 7];
        let in_fragmented_record = false;
        let record: Vec<u8> = vec![];
        let mut scratch: Vec<u8> = vec![];

        loop {
            self.read_physical_record(HEADER_SIZE, &mut header);
            let fragment_type = FragmentType::try_from(header[0]).unwrap();
            let length =  u16::from_le_bytes([header[1], header[2]]);


            match fragment_type {
                FragmentType::First => {

                }

                FragmentType::Middle => {
                    
                }

                FragmentType::Last => {

                }

                FragmentType::Full => {
                    self.read_physical_record(length as usize, &mut scratch);
                    return WalRecord::deserialize_record(&scratch).unwrap();
                }

            }
        }
        
    }


    fn read_physical_record(&mut self, length: usize, buffer: &mut [u8]) {
        let mut read = 0;


        while buffer.len() < length {
            read =  self.log_file.read(buffer).unwrap();

            if read == 0 {
                return 
            }

        }
    }
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
                    //TODO: fix log file methods use buf reader and write
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
        let fragment = FragmentHeader::new(fragment_type, length, record);

        //TODO: fix log file methods use buf reader and write
        // self.log_file.append(fragment.len(), &fragment);
        // self.log_file.append(length as usize, record);
        // self.log_file.flush();
        // self.block_offset += HEADER_SIZE + length as usize;
    }

    fn new(path: PathBuf) -> Self { todo!()}
    fn append(&mut self, length: usize, data: &[u8]) {
        
    }
    fn close(&self) {}

    fn flush(&self){}

    fn sync(&self){}

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

