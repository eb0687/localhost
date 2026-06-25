use std::collections::HashMap;
use std::mem;
use std::os::fd::RawFd;
use std::sync::Arc;
use std::time::{Duration, Instant};

use libc::{EPOLLIN, epoll_event};

use crate::conn::Conn;
use crate::handlers::error_response;
use crate::https::{HttpMethod, Request, Response, StatusCode};
use crate::info;
use crate::utils::helpers::create_epoll;
use crate::utils::helpers::{close_fd, create_listen_socket, epoll_add};

mod event_loop;
mod request_parsing;
mod route_matching;
mod session;

const IDLE_TIMEOUT_SECS: u64 = 10;
const IDLE_TIMEOUT: Duration = Duration::from_secs(IDLE_TIMEOUT_SECS);

const SESSION_TTL_SECS: u64 = 60 * 30;
const SESSION_TTL: Duration = Duration::from_secs(SESSION_TTL_SECS);

pub type Handler = Arc<dyn Fn(&Request, &Data) -> Response + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Data {
    pub path_value: HashMap<String, String>,
    pub query_value: HashMap<String, String>,
    pub session_id: Option<String>,
    pub is_new_session: bool,
    pub body: Vec<u8>,
}

pub struct Route {
    pub methods: Vec<HttpMethod>,
    pub pattern: String,
    pub handler: Handler,
}

pub struct VirtualServer {
    pub host: String,
    pub ports: Vec<u16>,
    pub server_names: Vec<String>,
    pub client_max_body_size: Option<usize>,
    pub error_pages: HashMap<u16, String>,
    pub routes: Vec<Route>,
}

pub struct Router {
    servers: Vec<VirtualServer>,
    default_server_by_port: HashMap<u16, usize>,
    server_name_by_port: HashMap<(u16, String), usize>,

    epfd: i32,
    conns: HashMap<RawFd, Conn>,
    events: Vec<epoll_event>,
    listen_fd_to_port: HashMap<RawFd, u16>,
    sessions: HashMap<String, Session>,
}

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub created_at: Instant,
    pub last_seen: Instant,
    pub visits: u64,
}

pub struct PendingRequest {
    pub header_bytes: Vec<u8>,
    pub body_bytes: Vec<u8>,
    pub local_port: u16,
}

pub enum ReadOutcome {
    Pending,
    Ready(PendingRequest),
    Error { status: StatusCode, reason: String },
}

impl Router {
    pub fn new_on_ports(ports: &[u16]) -> Self {
        let epfd = match create_epoll() {
            Ok(fd) => fd,
            Err(err) => {
                eprintln!("could not create epoll instance: {err}");
                -1
            }
        };
        let mut listen_fd_to_port: HashMap<RawFd, u16> = HashMap::new();

        for &port in ports {
            match create_listen_socket(port) {
                Ok(listen_fd) => {
                    info!("listening on 0.0.0.0:{port}");
                    if let Err(err) = epoll_add(epfd, listen_fd, EPOLLIN as u32)
                    {
                        eprintln!(
                            "could not register listener on port {port} in epoll: {err}"
                        );
                        close_fd(listen_fd);
                        continue;
                    }
                    listen_fd_to_port.insert(listen_fd, port);
                }
                Err(err) => {
                    println!(
                        "could not create a listener on port: {port}, error: {err}"
                    );
                }
            };
        }

        let conns: HashMap<RawFd, Conn> = HashMap::new();
        let events: Vec<epoll_event> = vec![unsafe { mem::zeroed() }; 128];

        Self {
            servers: Vec::new(),
            default_server_by_port: HashMap::new(),
            server_name_by_port: HashMap::new(),

            epfd,
            conns,
            events,
            listen_fd_to_port,
            sessions: HashMap::new(),
        }
    }

    pub fn add_virtual_server(&mut self, server: VirtualServer) {
        let server_index = self.servers.len();

        for &port in &server.ports {
            self.default_server_by_port
                .entry(port)
                .or_insert(server_index);

            for name in &server.server_names {
                self.server_name_by_port
                    .insert((port, name.to_ascii_lowercase()), server_index);
            }
        }

        self.servers.push(server);
    }

    pub(super) fn select_server_by_host(
        &self,
        local_port: u16,
        host_header: Option<&str>,
    ) -> Option<&VirtualServer> {
        if let Some(host_header) = host_header {
            let host = normalize_host_header(host_header);

            if let Some(server_index) =
                self.server_name_by_port.get(&(local_port, host))
            {
                return self.servers.get(*server_index);
            }
        }

        let default_index = self.default_server_by_port.get(&local_port)?;
        self.servers.get(*default_index)
    }

    fn select_server(
        &self,
        local_port: u16,
        req: &Request,
    ) -> Option<&VirtualServer> {
        self.select_server_by_host(local_port, req.headers.get("host"))
    }

    pub fn handle(&mut self, local_port: u16, req: &Request) -> Response {
        let match_result = {
            let Some(server) = self.select_server(local_port, req) else {
                return error_response(&req.version, StatusCode::NotFound);
            };

            let mut matched_path_but_wrong_method = false;
            let mut found: Option<(Handler, HashMap<String, String>)> = None;

            for route in &server.routes {
                let Some(path_value) =
                    route_matching::match_pattern(&route.pattern, &req.path)
                else {
                    continue;
                };

                if !route.methods.iter().any(|m| *m == req.method) {
                    matched_path_but_wrong_method = true;
                    continue;
                }

                found = Some((route.handler.clone(), path_value));
                break;
            }

            (found, matched_path_but_wrong_method)
        };

        let (found, matched_path_but_wrong_method) = match_result;
        let Some((handler, path_value)) = found else {
            if matched_path_but_wrong_method {
                return error_response(
                    &req.version,
                    StatusCode::MethodNotAllowed,
                );
            }
            return error_response(&req.version, StatusCode::NotFound);
        };

        let now = Instant::now();
        let (session_id, is_new_session) =
            session::resolve_session(&mut self.sessions, req, now);

        let data = Data {
            path_value,
            query_value: route_matching::parse_query(&req.query),
            session_id: session_id.clone(),
            is_new_session,
            body: req.data.body.clone(),
        };

        let mut resp = handler(req, &data);

        if is_new_session && let Some(sid) = session_id {
            let cookie = format!("sid={sid}; Path=/; HttpOnly; SameSite=Lax");
            resp.headers.insert("Set-Cookie", &cookie);
        }

        resp
    }

    pub fn listen_and_serve(&mut self) {
        loop {
            if let Err(err) = self.handle_connections() {
                eprintln!("server loop error: {err}");
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

fn normalize_host_header(host: &str) -> String {
    let host = host.trim();

    if let Some((name, _port)) = host.rsplit_once(':') {
        return name.to_ascii_lowercase();
    }

    host.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::normalize_host_header;

    #[test]
    fn normalizes_host_header_with_port() {
        assert_eq!(normalize_host_header("LOCALHOST:8080"), "localhost");
    }

    #[test]
    fn normalizes_host_header_without_port() {
        assert_eq!(normalize_host_header("Example.COM"), "example.com");
    }
}
