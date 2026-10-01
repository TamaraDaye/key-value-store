#![allow(unused)]
use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

struct Server {
    address: SocketAddr,
    view_server: SocketAddr, 
    role: Role
}

enum Role {
    Primary, 
    Backup
}

