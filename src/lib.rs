#![allow(unused)]
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::{HashMap};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};
use std::{thread, vec};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::{select, time};

type Server = SocketAddr;


enum ServerState {
    Missing,
    Alive,
    Dead,
}

pub struct ViewServer {
    view: Option<View>,
    address: SocketAddr,
    last_ping: HashMap<Server, Instant>,
    clients: Vec<Client>,
}

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
        let mut buf_reader = BufReader::new(stream);
        let mut response = String::new();
        buf_reader.read_line(&mut response);
        let view: View = serde_json::from_str(&response).unwrap();
        return view
    }
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
            });
        }

        self.last_ping.insert(server, Instant::now());
    }

    fn is_dead(&self, server: &Server, tick: tokio::time::Instant) -> bool {
        tick.saturating_duration_since(self.last_ping[&server].into()) > DEAD_TIMEOUT
    }

    fn validate_view(&mut self, tick: tokio::time::Instant) -> bool {
        let Some(view) = &self.view else {
            return false;
        };
        let primary_state = if self.is_dead(&view.primary, tick) {
            ServerState::Dead
        } else {
            ServerState::Alive
        };

        let backup_state = match view.backup {
            None => ServerState::Missing,
            Some(server) => {
                if self.is_dead(&server, tick) {
                    ServerState::Dead
                } else {
                    ServerState::Alive
                }
            }
        };

        let mut candidates: Vec<Server> = Vec::with_capacity(2);

        for (server, ping) in self.last_ping.iter() {
            if *server == view.primary
                || view.backup.as_ref().is_some_and(|backup| server == backup)
            {
                continue;
            }

            if tick.saturating_duration_since((*ping).into()) <= DEAD_TIMEOUT {
                candidates.push(*server);

                if candidates.len() == 2 {
                    break;
                }
            }
        }
        let Some(view) = &mut self.view else {
            return false;
        };

        return Self::update_view(view, primary_state, backup_state, candidates);
    }

    fn update_view(
        view: &mut View,
        primary_state: ServerState,
        backup_state: ServerState,
        candidate_servers: Vec<Server>,
    ) -> bool {
        match (primary_state, backup_state) {
            (ServerState::Alive, ServerState::Missing) => {
                if let Some(server) = candidate_servers.first() {
                    view.view_number += 1;
                    view.backup = Some(*server);
                    true
                } else {
                    false
                }
            }

            (ServerState::Alive, ServerState::Dead) => {
                if let Some(server) = candidate_servers.first() {
                    view.view_number += 1;
                    view.backup = Some(*server);
                    true
                } else {
                    eprintln!("There are no idle servers for the backup");
                    false
                }
            }
            (ServerState::Dead, ServerState::Alive) => {
                view.view_number += 1;
                view.primary = view.backup.unwrap();
                if let Some(server) = candidate_servers.first() {
                    view.backup = Some(*server)
                } else {
                    view.backup = None
                }

                true
            }

            (ServerState::Dead, ServerState::Missing) | (ServerState::Dead, ServerState::Dead) => {
                eprintln!("There's no new primary twin");
                false
            }

            _ => false,
        }
    }

    pub async fn handle_connection(&self, stream: &mut TcpStream) -> Response {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let request: Request = serde_json::from_str(&line).unwrap();
        match request {
            Request::Ping => { 
                todo!()
            }
            Request::Discover => { todo!() }
        }

        Response::Ack
    }

    fn find_idle_server(&self, dead_servers: &HashSet<Server>) -> Server {
        todo!()
    }

    async fn update_client(&mut self) {
        todo!()
    }
}

pub async fn run(view_server: &mut ViewServer) {
    let server: TcpListener = TcpListener::bind(view_server.address).await.unwrap();

    let mut interval = time::interval(DEAD_TIMEOUT);

    loop {
        select! {
        last_tick = interval.tick() => {
            println!("Ticker fired i guess");
            let view_changed = view_server.validate_view(last_tick);
        }

        conn = server.accept() => {
            match conn {
                Ok((mut stream, addr)) => {
                    view_server.handle_connection(&mut stream).await;
                },
                Err(e) => todo!()
            };
        }
        }
    }
}
