#![allow(unused)]
pub mod server;
pub mod storage_engine;
pub mod wal;
pub mod sst;

use std::path::PathBuf;
use crc32c::crc32c;

struct DbPaths {
    root: PathBuf,
    wal_dir: PathBuf,
    sst_dir: PathBuf,
}

impl DbPaths {
    fn new(root: PathBuf) -> Self {
        Self {
            wal_dir: root.join("wal"),
            sst_dir: root.join("sst"),
            root,
        }
    }
}

