use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::Deserialize;

use crate::https::HttpMethod;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema: Option<String>,
    pub servers: Vec<ServerConfig>,
}

impl Config {
    pub fn listener_ports(&self) -> Result<Vec<u16>, String> {
        let mut unique_ports: HashSet<u16> = HashSet::new();

        for server in &self.servers {
            for &port in &server.ports {
                unique_ports.insert(port);
            }
        }

        if unique_ports.is_empty() {
            return Err("config has no listener ports".to_string());
        }

        let mut ports: Vec<u16> = unique_ports.into_iter().collect();
        ports.sort_unstable();
        Ok(ports)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub host: String,
    pub ports: Vec<u16>,
    pub server_name: Vec<String>,
    pub client_max_body_size: Option<usize>,
    pub error_pages: Option<HashMap<String, String>>,
    pub routes: Vec<RouteRule>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum RouteRule {
    FileServer(FileServerConfig),
    Cgi(CgiConfig),
    Redirect(RedirectConfig),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileServerConfig {
    pub path: String,
    pub root: String,
    pub allowed_methods: Option<Vec<HttpMethod>>,
    pub directory_listing: Option<bool>,
    pub default_file: Option<String>,

    #[serde(skip)]
    pub mount_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CgiConfig {
    pub path: String,
    pub root: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedirectConfig {
    pub path: String,
    pub target: String,
}

#[derive(Debug)]
pub struct AppConfig {
    pub config_path: PathBuf,
    pub config: Config,
    pub listener_ports: Vec<u16>,
}
