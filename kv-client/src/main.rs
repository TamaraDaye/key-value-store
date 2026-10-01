#![allow(unused)]
use tokio::net::{TcpStream, TcpListener};
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use protocol::{Request, Response, View, Server};

pub struct Client {
    view_server: Server,
    current_view: Option<View>,
}

impl Client {
    fn new(address: SocketAddr) -> Client {
        Client {
            view_server: address,
            current_view: None,
        }
    }

    fn put(key: String, value: String) -> (String, String) {
        todo!()
    }

    async fn discover(&self)-> View {
        let mut stream: TcpStream = TcpStream::connect(self.view_server).await.unwrap();
        let request: Request= Request::Discover;
        let data: Vec<u8> = serde_json::to_vec(&request).unwrap();
        stream.write_all(&data).await.unwrap();
        stream.write_all(b"\n");
        let mut buf_reader = BufReader::new(stream);
        let mut response = String::new();
        buf_reader.read_line(&mut response);
        let view: View = serde_json::from_str(&response).unwrap();
        return view
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
