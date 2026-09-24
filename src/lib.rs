#![allow(unused)]
use core::panic::PanicInfo;
use std::arch::x86_64::_CMP_FALSE_OQ;
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
        let Some(view) = &mut self.view else {
            return false;
        };
        let primary_dead =
            tick.saturating_duration_since(self.last_ping[&view.primary].into()) > DEAD_TIMEOUT;
        let backup_dead = view.backup.is_some_and(|server| {
            tick.saturating_duration_since(self.last_ping[&server].into()) > DEAD_TIMEOUT
        });

        if !primary_dead && !backup_dead {
            return false;
        }

        let mut candidates: Vec<Server> = Vec::with_capacity(2);

        for (server, ping) in self.last_ping.iter() {
            if *server == view.primary
                || view.backup.as_ref().is_some_and(|backup| server == backup)
            {
                continue;
            }

            if tick.saturating_duration_since((*ping).into()) <= DEAD_TIMEOUT {
                if candidates.len() == 2 {
                    break;
                }
                candidates.push(*server);
            }
        }

        Self::update_view(view, primary_dead, backup_dead, candidates);

        true
    }

    fn update_view(
        view: &mut View,
        primary_state: bool,
        backup_state: bool,
        candidate_servers: Vec<Server>,
    ) {
        match (primary_state, backup_state) {
            (true, false) => {
                view.view_number += 1;
                view.primary = view.backup.unwrap();
                view.backup = Some(candidate_servers[0]);
            }

            (false, true) => {
                view.view_number += 1;
                view.backup = Some(candidate_servers[0]);
            }

            _ => return
        }
    }

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
