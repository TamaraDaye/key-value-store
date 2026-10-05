#![allow(unused)]
use core::panic;
use protocol::{DatabaseStub, Request, Response, Server, View};
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

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

    async fn connect_to_db(&self) -> TcpStream {
        let Some(ref db) = self.current_view else {
            panic!("no view ")
        };
        let mut stream = TcpStream::connect(db.primary).await.unwrap();
        stream
    }

    async fn put(&self, key: String, value: String) -> (String, String) {
        let mut connection : TcpStream = self.connect_to_db().await;
        let data = DatabaseStub::Put(key, value);
        let mut request: Vec<u8> = serde_json::to_vec(&data).unwrap();
        request.push(b'\n');
        connection.write_all(&request).await.unwrap();
        todo!()
    }

    async fn get(&self, key: String) -> String {
        let mut connection : TcpStream = self.connect_to_db().await;
        let data = DatabaseStub::Get(key);
        let mut request: Vec<u8> = serde_json::to_vec(&data).unwrap();
        request.push(b'\n');
        connection.write_all(&request).await.unwrap();
        let mut buf_reader = BufReader::new(connection);
        let mut response = String::new();
        buf_reader.read_line(&mut response);
        return response
    }

    async fn append(&self, key: String, value: String) {
        let mut connection : TcpStream = self.connect_to_db().await;
        let data = DatabaseStub::Append(key, value);
        let mut request: Vec<u8> = serde_json::to_vec(&data).unwrap();
        request.push(b'\n');
        connection.write_all(&request).await.unwrap();
        todo!()
    }

    async fn discover(&self) -> View {
        let mut connection : TcpStream = self.connect_to_db().await;
        let data = Request::Discover;
        let mut request: Vec<u8> = serde_json::to_vec(&data).unwrap();
        request.push(b'\n');
        connection.write_all(&request).await.unwrap();
        let mut buf_reader = BufReader::new(connection);
        let mut response = String::new();
        buf_reader.read_line(&mut response);
        let view: View = serde_json::from_str(&response).unwrap();
        return view;
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
