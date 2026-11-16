#![allow(unused)]

//TODO: use batched writes to make writes efficient and faster can afford buf writer
//
use crate::{DbPaths, storage_engine::Writer};
use crc32c;
use protocol::BinCodec;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use std::fmt::Write;
use std::fs::File;
use std::io::{BufReader, BufWriter, IoSlice, Read};
use std::ops::Range;
use std::os::fd;
use std::path::PathBuf;
use std::sync::mpsc;
use tokio::io::AsyncWriteExt;

const BLOCK_SIZE: usize = 32 << 10;
const SEGMENT_SIZE: usize = 16 << 20;
const HEADER_SIZE: usize = 7;
const MAX_RECORD_SIZE: usize = 8 << 20;
const PADDING: [u8; HEADER_SIZE] = [0u8; HEADER_SIZE];

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
        header[0] = fragment as u8;
        header[1..3].copy_from_slice(&record_length.to_le_bytes());
        header[3..].copy_from_slice(&checksum.to_le_bytes());

        header
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentType {
    Full = 1,
    First = 2,
    Middle = 3,
    Last = 4,
}

impl TryFrom<u8> for FragmentType {
    type Error = u8;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(FragmentType::Full),
            2 => Ok(FragmentType::First),
            3 => Ok(FragmentType::Middle),
            4 => Ok(FragmentType::Last),
            invalid => Err(invalid),
        }
    }
}

impl BinCodec for Operation {}

#[derive(Serialize, Deserialize)]
pub enum Operation {
    Put { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
    Create { column: String },
    Append,
    Write,
    SingleDelete,
}

pub struct WalWriter<W: Writer> {
    log_file: W,
    block_offset: usize,
}

pub struct WalReader<R: Read> {
    log_file: BufReader<R>,
    block: [u8; BLOCK_SIZE],
    block_size: usize,
    block_offset: usize,
    last_record_offset: usize,
    record_buffer: Vec<u8>,
}

//TODO: implement iterator for wal

impl<R: Read> WalReader<R> {
    pub fn read_record(&mut self) -> Option<WalRecord> {
        let mut in_fragmented_record = false;
        self.record_buffer.clear();

        loop {
            let (header, range) = self.read_physical_record()?;
            let payload_length = range.len();

            let fragment_type = FragmentType::try_from(header[0]).ok()?;
            let expected_crc = u32::from_le_bytes([header[3], header[4], header[5], header[6]]);

            let actual_crc = generate_checksum(fragment_type, &self.block[range.clone()]).ok()?;
            if expected_crc != actual_crc {
                return None;
            }

            match fragment_type {
                FragmentType::First => {
                    if in_fragmented_record {
                        return None;
                    }
                    in_fragmented_record = true;
                    self.record_buffer.extend_from_slice(&self.block[range]);
                }

                FragmentType::Middle => {
                    if !in_fragmented_record {
                        return None;
                    }
                    self.record_buffer.extend_from_slice(&self.block[range]);
                }

                FragmentType::Last => {
                    if !in_fragmented_record {
                        return None;
                    }
                    self.record_buffer.extend_from_slice(&self.block[range]);
                    self.last_record_offset += HEADER_SIZE + payload_length;
                    return WalRecord::deserialize_record(&self.record_buffer).ok();
                }

                FragmentType::Full => {
                    if in_fragmented_record {
                        return None;
                    }
                    self.last_record_offset += HEADER_SIZE + payload_length;
                    return WalRecord::deserialize_record(&self.block[range]).ok();
                }
            }
        }
    }

    fn read_physical_record(&mut self) -> Option<([u8; HEADER_SIZE], Range<usize>)> {
        loop {
            if self.block_size - self.block_offset <= HEADER_SIZE {
                if !self.read_block() {
                    return None;
                }
            }

            let mut header = [0u8; HEADER_SIZE];
            header.copy_from_slice(&self.block[self.block_offset..self.block_offset + HEADER_SIZE]);

            let length = u16::from_le_bytes([header[1], header[2]]) as usize;
            let payload_start = self.block_offset + HEADER_SIZE;
            let payload_end = payload_start + length;

            //This check is for corrupted writes in the case where the last
            //segment or block written to isn't a full BLOCK_SIZE and only the header of the block
            //was successfully written to the disk page which could indicate a corrupted block
            if payload_end > self.block_size {
                return None;
            }

            self.block_offset = payload_end;
            return Some((header, payload_start..payload_end));
        }
    }

    fn read_block(&mut self) -> bool {
        self.block_size = 0;
        self.block_offset = 0;

        while self.block_size < BLOCK_SIZE {
            match self.log_file.read(&mut self.block[self.block_size..]) {
                Ok(0) => break,
                Ok(n) => self.block_size += n,
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => return false,
            }
        }

        self.block_size > 0
    }
}

impl<R: Read> Iterator for WalReader<R> {
    type Item = WalRecord;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.read_record()
    }
}

impl<W: Writer> WalWriter<W> {
    fn add_record(&mut self, record: &[u8]) -> Result<(), String> {
        let mut ptr = 0;
        let mut left = record.len();

        let mut begin = true;

        while left > 0 {
            let leftover = BLOCK_SIZE - self.block_offset;

            if leftover <= HEADER_SIZE {
                if (leftover > 0) {
                    let padding = &PADDING[..leftover];
                    self.append(&padding);
                }

                self.block_offset = 0
            }

            let avail = BLOCK_SIZE - self.block_offset - HEADER_SIZE;

            let fragment_length = left.min(avail);

            let end = left == fragment_length;

            let mut fragment_type: FragmentType;

            if begin && end {
                fragment_type = FragmentType::Full;
            } else if begin {
                fragment_type = FragmentType::First
            } else if end {
                fragment_type = FragmentType::Last
            } else {
                fragment_type = FragmentType::Middle;
            }

            self.emit_record(
                fragment_type,
                &record[ptr..ptr + fragment_length],
                fragment_length as u16,
            );

            left -= fragment_length;
            ptr += fragment_length;
            begin = false;
        }
        Ok(())
    }

    fn emit_record(&mut self, fragment_type: FragmentType, record: &[u8], length: u16) {
        let fragment = FragmentHeader::new(fragment_type, length, record);
        self.append(&fragment);
        self.append(&record);
        self.block_offset += HEADER_SIZE + length as usize;
    }

    fn append(&mut self, data: &[u8]) {
        self.log_file.write_all(&data).unwrap();
    }

    fn sync(&mut self) {
        self.log_file.flush();
    }
}

pub struct Wal<W: Writer> {
    blocks: u32,
    record_queue: mpsc::Receiver<Operation>,
    writer: WalWriter<W>,
}

#[derive(Serialize, Deserialize)]
pub struct WalRecord {
    sequence_number: u64,
    operation: Operation,
}

pub struct PhysicalRecord {
    length: [u8; 2],
    payload: [u8],
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


#[cfg(test)]
mod tests {
    use super::*;

    fn test_writer(){}


    fn test_reader(){}


    fn test_fragmentation(){}

    fn test_truncated_writes(){}
}
