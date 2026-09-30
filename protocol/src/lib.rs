#![allow(unused)]
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

type Server = SocketAddr;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    Discover,
    Ping,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Response {
    View(View),
    Ack,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    view_number: u32,
    primary: Server,
    backup: Option<Server>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Procedure {
    Response,
    Request,
}
