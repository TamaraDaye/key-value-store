#![allow(unused)]
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{collections::HashMap, net::SocketAddr};

pub trait JsonCodec: Serialize + DeserializeOwned + Sized {
    fn serialize(&self) -> serde_json::Result<Vec<u8>> {
        let mut data = serde_json::to_vec(self)?;
        data.push(b'\n');
        Ok(data)
    }
    fn deserialize(bytes: &[u8]) -> serde_json::Result<Self> {
        serde_json::from_slice(bytes)
    }
}

pub trait BinCodec: Serialize + DeserializeOwned {
    fn serialize_record(&self) -> postcard::Result<Vec<u8>> {
         postcard::to_allocvec(self)
    }

    fn deserialize_record(bytes: &[u8]) -> postcard::Result<Self> 
    where Self: Sized,
    {
        postcard::from_bytes(bytes)
    }
}

impl<T: Serialize + DeserializeOwned> JsonCodec for T {}

pub type Server = SocketAddr;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    Discover,
    Ping { server: Server, view: u32 },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    View(Option<View>),
    Ack,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    pub view_number: u32,
    pub primary: Server,
    pub backup: Option<Server>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Procedure {
    Response,
    Request,
}

//certain operations should be atomic e.g writes
#[derive(Serialize, Deserialize)]
pub enum DatabaseStub {
    Create(String),
    Put(String, String),
    Get(String),
    Append(String, String),
    Write(HashMap<String, String>), //atomic writes,
    MultiGet(Vec<String>),
    SingleDelete(String),
    NewIterator,
    Snapshot,
    CreateColumn(String), //serves as a namespace
}
