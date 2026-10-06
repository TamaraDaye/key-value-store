#![allow(unused)]
use serde::{Serialize,  de::DeserializeOwned, Deserialize};
use std::net::SocketAddr;

pub trait SerdeData: Serialize + DeserializeOwned + Sized {
    fn serialize_request(&self) -> serde_json::Result<Vec<u8>> { 
        let mut request = serde_json::to_vec(self)?;
        request.push(b'\n');
        Ok(request)
    }
    fn deserialize_request(bytes: &[u8]) -> serde_json::Result<Self>{ 
        serde_json::from_slice(bytes)
    }
}

impl<T: Serialize + DeserializeOwned> SerdeData for T {}

pub type Server = SocketAddr;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    Discover,
    Ping{view: usize},
}


#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    View(View),
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

