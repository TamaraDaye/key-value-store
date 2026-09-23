#![allow(unused)]
use core::panic::PanicInfo;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};
use std::{thread, vec};
use tokio::net::{TcpListener, TcpStream};
use tokio::{select, time};
use std::cell::RefCell;

type Server = SocketAddr;

const PING_INTERVAL: Duration = Duration::from_millis(100);
const DEAD_PINGS: u32 = 3;
const DEAD_TIMEOUT: Duration = PING_INTERVAL.saturating_mul(DEAD_PINGS);
const VIEWS: Vec<View> = vec![];

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
    view: Option<View>,
    address: SocketAddr,
    last_ping: HashMap<Server, Instant>,
}

impl ViewServer {
    fn new() -> ViewServer {
        let address: SocketAddr = "127.0.0.1:6479".parse().unwrap();

        ViewServer {
            view: None,
            address,
            last_ping: HashMap::new(),
        }
    }

    fn record_request(&mut self, server: Server) {
        self.last_ping.insert(server, Instant::now());
    }

    fn validate_view(&mut self, tick: Instant) {
        let dead_servers: HashSet<Server> = self.last_ping
            .iter()
            .filter(|(_, t)| tick.saturating_duration_since(**t) > DEAD_TIMEOUT)
            .map(|(server, _)| *server)
            .collect();

        if dead_servers.is_empty() {
            return;
        }

        self.update_view(&dead_servers);
    }

    fn update_view(&mut self, dead_servers: &HashSet<Server>) { 
    }


    fn is_server_in_view(&self, server: &Server) -> bool {
        todo!()
    }

    fn find_idle_server(&self, dead_servers: &HashSet<Server>) -> Server {
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
            last_tick = interval.tick() => {
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
