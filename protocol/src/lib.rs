#![allow(unused)]
use serde::{Serialize,  de::DeserializeOwned, Deserialize};
use std::net::SocketAddr;

pub trait SerdeData: Serialize + DeserializeOwned + Sized {
    fn serialize(&self) -> serde_json::Result<Vec<u8>> { 
        let mut data = serde_json::to_vec(self)?;
        data.push(b'\n');
        Ok(data)
    }
    fn deserialize(bytes: &[u8]) -> serde_json::Result<Self>{ 
        serde_json::from_slice(bytes)
    }
}

impl<T: Serialize + DeserializeOwned> SerdeData for T {}

pub type Server = SocketAddr;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    Discover,
    Ping{server: Server, view: u32},
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

#[derive(Serialize, Deserialize)]
pub enum DatabaseStub {
    Put(String, String),
    Get(String),
    Append(String, String),
}

