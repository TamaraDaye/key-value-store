#![allow(unused)]
use protocol::{Request, Response, SerdeData, Server, View};
use std::collections::HashMap;
use std::error;
use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Instant, interval};

use tokio::sync::{mpsc, oneshot};

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

enum ActorEvent {
    Tick(Instant),
    Rpc {
        server: Server,
        request: Request,
        reply_tx: oneshot::Sender<Response>,
    },
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

    async fn update_client(&mut self) {
        todo!()
    }
}

#[tokio::main]
async fn main() {}

pub async fn run_actor(view_server: &mut ViewServer, mut rx: mpsc::Receiver<ActorEvent>) {
    while let Some(event) = rx.recv().await {
        match event {
            ActorEvent::Tick(now) => {
                view_server.process_view(now);
            }

            ActorEvent::Rpc {
                server,
                request,
                reply_tx,
            } => {
                let response = match request {
                    Request::Ping { server, view } => {
                        view_server.record_request(server, view);
                        view_server.process_view(Instant::now());
                        Response::View(view_server.view.clone())
                    }

                    Request::Discover => Response::View(view_server.view.clone()),
                };

                let _ = reply_tx.send(response);
            }
        }
    }
}

pub async fn run_timer(tx: mpsc::Sender<ActorEvent>) {
    let mut ticker = interval(PING_INTERVAL);

    loop {
        let now = ticker.tick().await;
        if tx.send(ActorEvent::Tick(now)).await.is_err() {
            break;
        }
    }
}

async fn run_network(listener: TcpListener, tx: mpsc::Sender<ActorEvent>) {
    while let Ok((mut stream, addr)) = listener.accept().await {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();

            if reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                if let Ok(request) = serde_json::from_str::<Request>(&line) {
                    let (reply_tx, reply_rx) = oneshot::channel();

                    let event = ActorEvent::Rpc { server: addr, request, reply_tx };

                    if tx.send(event).await.is_ok() {
                        if let Ok(response) = reply_rx.await {
                            if let Ok(bytes) = response.serialize() {
                                let _ = stream.write_all(&bytes).await;
                            }
                        }
                    }
                };
            }
        });
    }
}
