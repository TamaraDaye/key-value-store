#![allow(unused)]

use crate::storage_engine::Writer;

pub struct SSTable<W: Writer> {
    table: W
}


pub struct TableMetaData {
    data_size : usize, 
    idex_size: usize, 
    raw_key_size: usize, 
    number_of_entries: usize , 
    data_blocks: usize
}
