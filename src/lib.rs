#![allow(unused)]
use core::panic::PanicInfo;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
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

#[derive(Clone, Debug)]
pub struct View {
    view_number: u32,
    primary: Server,
    backup: Option<Server>,
}

pub struct ViewServer {
    view: Option<View>,
    address: SocketAddr,
    last_ping: HashMap<Server, Instant>,
    clients: Vec<Client>,
}

pub struct Client {
    addr: SocketAddr,
}

impl ViewServer {
    fn new() -> ViewServer {
        let address: SocketAddr = "127.0.0.1:6479".parse().unwrap();

        ViewServer {
            view: None,
            address,
            last_ping: HashMap::new(),
            clients: vec![],
        }
    }

    fn record_request(&mut self, server: Server) {
        if self.view.is_none() {
            self.view = Some(View {
                view_number: 1,
                primary: server,
                backup: None,
            })
        }
        self.last_ping.insert(server, Instant::now());
    }

    fn validate_view(&mut self, tick: tokio::time::Instant) -> bool {
        for (server, t) in self.last_ping.iter() {
            if tick.saturating_duration_since((*t).into()) > DEAD_TIMEOUT {

            }
        }

        true
    }

    // fn update_view(&mut self, dead_servers: &HashSet<Server>) {
    //     let Some(view) = &mut self.view else { return };

    //     let primary_dead = dead_servers.contains(&view.primary);
    //     let secondary_dead = dead_servers.contains(&view.backup);

    //     match (primary_dead, secondary_dead) {
    //         (true, true) => {
    //             todo!()
    //         }

    //         (true, false) => {
    //             view.primary = view.backup;
    //             view.backup = todo!();
    //             view.view_number += 1;
    //         }
    //         (false, true) => {
    //             todo!()
    //         }
    //         (false, false) => {
    //             return;
    //         }
    //     }
    // }

    fn find_idle_server(&self, dead_servers: &HashSet<Server>) -> Server {
        todo!()
    }

    async fn update_client(&mut self) {
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
               let valid_view = view_server.validate_view(last_tick);
            if !valid_view {
                view_server.update_client();
            }
        }

            conn = server.accept() => {
                match conn {
                    Ok((stream, addr)) => {
                        parse_request(&stream).await;
                        view_server.record_request(addr);
                    },
                    Err(e) => todo!()
                };
            }
        }
    }
}

pub async fn parse_request(stream: &TcpStream) {}
