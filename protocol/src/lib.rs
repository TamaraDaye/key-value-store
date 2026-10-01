#![allow(unused)]
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub type Server = SocketAddr;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    Discover,
    Ping,
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
