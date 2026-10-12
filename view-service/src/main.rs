#![allow(unused)]
use protocol::{Request, Response, SerdeData, Server, View};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::time::{Instant, interval};
use thiserror::Error;
use std::error;


const PING_INTERVAL: Duration = Duration::from_millis(100);
const DEAD_PINGS: u32 = 3;
const DEAD_TIMEOUT: Duration = PING_INTERVAL.saturating_mul(DEAD_PINGS);
const TICK_INTERVAL: Duration = Duration::from_millis(100);


#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("failed to read request")]
    Read(#[source] std::io::Error),

    #[error("malformed request")]
    BadRequest(#[source] serde_json::Error),

    #[error("failed to serialize response")]
    Serialize(#[source] serde_json::Error), // adjust to whatever .serialize() returns

    #[error("failed to write response")]
    Write(#[source] std::io::Error),
}

enum ServerState {
    Missing,
    Alive,
    Dead,
}

pub struct ViewServer {
    view: Option<View>,
    address: SocketAddr,
    last_ping: HashMap<Server, Instant>,
    acked_view: Option<u32>,
}

impl ViewServer {
    fn new(ip: SocketAddr) -> ViewServer {

        ViewServer {
            view: None,
            address: ip,
            last_ping: HashMap::new(),
            acked_view: None,
        }
    }

    fn record_request(&mut self, server: Server, view_num: u32) {
        if let Some(view) = &self.view {
            if server == view.primary && view_num == view.view_number {
                self.acked_view = Some(view_num)
            }
        } else {
            self.view = Some(View {
                view_number: 1,
                primary: server,
                backup: None,
            });
        }

        self.last_ping.insert(server, Instant::now());
    }

    fn is_dead(&self, server: &Server, tick: Instant) -> bool {
        match self.last_ping.get(server) {
            Some(last_ping) => tick.saturating_duration_since((*last_ping).into()) > DEAD_TIMEOUT,
            None => true,
        }
    }
    fn process_view(&mut self, tick: tokio::time::Instant) -> bool {
        let (primary_state, backup_state, candidates) = {
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
            (primary_state, backup_state, candidates)
        };

        let Some(view) = &mut self.view else {
            return false;
        };

        if self.acked_view != Some(view.view_number) {
            return false;
        }

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

    pub async fn handle_connection(&mut self, stream: &mut TcpStream, server: Server) -> Result<(), NetworkError>{
        let mut reader = BufReader::new(&mut *stream);
        let mut line = String::new();
        let n = reader.read_line(&mut line).await.map_err(NetworkError::Read)?;
        if n == 0 {
            return Ok(())
        }

        let request: Request = serde_json::from_str(&line).map_err(NetworkError::BadRequest)?;

        match request {
            Request::Ping { view } => {
                self.record_request(server, view);
                self.process_view(Instant::now());
                let response = Response::View(self.view.clone()).serialize().map_err(NetworkError::Serialize)?;
                stream.write_all(&response).await.map_err(NetworkError::Write);
            }
            Request::Discover => {
                let response = Response::View(self.view.clone()).serialize().unwrap();
                stream.write_all(&response).await;
                stream.write_all(&response).await.map_err(NetworkError::Write);
            }
        }

        Ok(())
    }

    async fn update_client(&mut self) {
        todo!()
    }
}

#[tokio::main]
async fn main() {
}

pub async fn run(view_server: &mut ViewServer) {
    let server: TcpListener = TcpListener::bind(view_server.address).await.unwrap();

    let mut interval = interval(PING_INTERVAL);

    loop {
        select! {
        last_tick = interval.tick() => {
            println!("Ticker fired i guess");
            let view_changed = view_server.process_view(last_tick);
        }

        conn = server.accept() => {
            match conn {
                Ok((mut stream, addr)) => {
                    view_server.handle_connection(&mut stream, addr).await;
                },
                Err(e) => todo!()
            };
        }
        }
    }
}
