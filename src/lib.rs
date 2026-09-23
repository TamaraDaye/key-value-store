#![allow(unused)]
use core::panic::PanicInfo;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};
use std::{thread, vec};
use tokio::net::{TcpListener, TcpStream};
use tokio::{select, time};

type Server = SocketAddr;

const PING_INTERVAL: Duration = Duration::from_millis(100);
const DEAD_PINGS: u32 = 3;
const DEAD_TIMEOUT: Duration = PING_INTERVAL.saturating_mul(DEAD_PINGS);

pub enum Procedures {
    Get(String),
    Put(String, String),
    Append(String, String),
}

pub struct View {
    view_number: u32,
    primary: Server,
    secondary: Server,
    backup: Server,
}

pub struct ViewServer {
    views: Option<Vec<View>>,
    address: SocketAddr,
    last_ping: HashMap<Server, Instant>,
}

impl ViewServer {
    fn new() -> ViewServer {
        let address: SocketAddr = "127.0.0.1:6479".parse().unwrap();

        ViewServer {
            views: None,
            address,
            last_ping: HashMap::new(),
        }
    }

    fn update_view(&mut self) {}

    fn record_request(&mut self, server: Server) {
        self.last_ping.insert(server, Instant::now());
    }

    fn validate_view(&mut self) {

    }

    fn check_ping(&self, addr: SocketAddr) {
        todo!()
    }
}

pub struct Rpc {
    procedure: Procedures,
}

pub async fn discover(view_server: &mut ViewServer) {
    let server: TcpListener = TcpListener::bind(view_server.address).await.unwrap();

    let mut interval = time::interval(DEAD_TIMEOUT);

    loop {
        select! {
            _ = interval.tick() => {
                println!("Ticker fired i guess");
            }

            conn = server.accept() => {
                match conn {
                    Ok((_stream, addr)) => {
                        view_server.record_request(addr);
                    },
                    Err(e) => todo!()
                };
            }
        }
    }
}

pub async fn parse_request(stream: &TcpStream) {}
