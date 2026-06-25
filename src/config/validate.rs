use std::collections::{HashMap, HashSet};

use crate::config::model::{
    CgiConfig, Config, FileServerConfig, RedirectConfig, RouteRule,
    ServerConfig,
};
use crate::utils::helpers::normalize_path;

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        let mut errors: Vec<String> = Vec::new();

        if self.servers.is_empty() {
            errors.push("config has no servers".to_string());
        }

        validate_server_names(self, &mut errors);

        for (server_index, server) in self.servers.iter().enumerate() {
            validate_server(server_index, server, &mut errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let mut out = String::from("config validation failed:\n");
            for err in errors {
                out.push_str(" - ");
                out.push_str(&err);
                out.push('\n');
            }
            Err(out)
        }
    }
}

fn validate_server_names(config: &Config, errors: &mut Vec<String>) {
    let mut seen: HashMap<(u16, String), usize> = HashMap::new();

    for (server_index, server) in config.servers.iter().enumerate() {
        for &port in &server.ports {
            for name in &server.server_name {
                let normalized = name.trim().to_ascii_lowercase();

                if normalized.is_empty() {
                    errors.push(format!(
                        "server {server_index}: server_name cannot contain empty names"
                    ));
                    continue;
                }

                let key = (port, normalized);
                if let Some(first_server_index) = seen.insert(key, server_index)
                {
                    errors.push(format!(
                        "server {server_index}: duplicate server_name '{name}' on port {port}; already used by server {first_server_index}"
                    ));
                }
            }
        }
    }
}

fn validate_server(
    server_index: usize,
    server: &ServerConfig,
    errors: &mut Vec<String>,
) {
    if server.host.trim().is_empty() {
        errors.push(format!("server {server_index}: host is required"));
    }

    if server.ports.is_empty() {
        errors.push(format!(
            "server {server_index}: ports must contain at least one port"
        ));
    }

    let mut seen_ports = HashSet::new();
    for &port in &server.ports {
        if port == 0 {
            errors.push(format!(
                "server {server_index}: port must be in 1..=65535"
            ));
        }

        if !seen_ports.insert(port) {
            errors.push(format!(
                "server {server_index}: duplicate port {port} in same server"
            ));
        }
    }

    if let Some(limit) = server.client_max_body_size {
        if limit == 0 {
            errors.push(format!(
                "server {server_index}: client_max_body_size must be > 0"
            ));
        }
    }

    validate_error_pages(server_index, server, errors);

    if server.routes.is_empty() {
        errors.push(format!(
            "server {server_index}: routes must contain at least one route"
        ));
    }

    let mut seen_routes = HashSet::new();
    for (route_index, route) in server.routes.iter().enumerate() {
        let path = route_path(route);

        if !path.starts_with('/') {
            errors.push(format!(
                "server {server_index}, route {route_index}: path must start with '/'"
            ));
        }

        let normalized_path = normalize_path(path);
        if !seen_routes.insert(normalized_path.clone()) {
            errors.push(format!(
                "server {server_index}, route {route_index}: duplicate route path '{normalized_path}'"
            ));
        }

        match route {
            RouteRule::FileServer(cfg) => {
                validate_file_server(server_index, route_index, cfg, errors);
            }
            RouteRule::Cgi(cfg) => {
                validate_cgi(server_index, route_index, cfg, errors);
            }
            RouteRule::Redirect(cfg) => {
                validate_redirect(server_index, route_index, cfg, errors);
            }
        }
    }
}

fn validate_error_pages(
    server_index: usize,
    server: &ServerConfig,
    errors: &mut Vec<String>,
) {
    let Some(error_pages) = &server.error_pages else {
        return;
    };

    for code_str in error_pages.keys() {
        match code_str.parse::<u16>() {
            Ok(code) if (400..=599).contains(&code) => {}
            Ok(code) => errors.push(format!(
                "server {server_index}: error_pages key '{code}' must be in 400..=599"
            )),
            Err(_) => errors.push(format!(
                "server {server_index}: error_pages key '{code_str}' is not a valid status code"
            )),
        }
    }
}

fn route_path(route: &RouteRule) -> &str {
    match route {
        RouteRule::FileServer(cfg) => &cfg.path,
        RouteRule::Cgi(cfg) => &cfg.path,
        RouteRule::Redirect(cfg) => &cfg.path,
    }
}

fn validate_file_server(
    server_index: usize,
    route_index: usize,
    cfg: &FileServerConfig,
    errors: &mut Vec<String>,
) {
    if cfg.path.trim().is_empty() {
        errors.push(format!(
            "server {server_index}, route {route_index} (file_server): path is required"
        ));
    }

    if cfg.root.trim().is_empty() {
        errors.push(format!(
            "server {server_index}, route {route_index} (file_server): root is required"
        ));
    }

    if let Some(methods) = &cfg.allowed_methods {
        if methods.is_empty() {
            errors.push(format!(
                "server {server_index}, route {route_index} (file_server): allowed_methods cannot be empty"
            ));
        }
    }

    if let Some(default_file) = &cfg.default_file {
        if default_file.trim().is_empty() {
            errors.push(format!(
                "server {server_index}, route {route_index} (file_server): default_file cannot be empty"
            ));
        }
    }
}

fn validate_cgi(
    server_index: usize,
    route_index: usize,
    cfg: &CgiConfig,
    errors: &mut Vec<String>,
) {
    if cfg.path.trim().is_empty() {
        errors.push(format!(
            "server {server_index}, route {route_index} (cgi): path is required"
        ));
    }

    if cfg.root.trim().is_empty() {
        errors.push(format!(
            "server {server_index}, route {route_index} (cgi): root is required"
        ));
    }
}

fn validate_redirect(
    server_index: usize,
    route_index: usize,
    cfg: &RedirectConfig,
    errors: &mut Vec<String>,
) {
    if cfg.path.trim().is_empty() {
        errors.push(format!(
            "server {server_index}, route {route_index} (redirect): path is required"
        ));
    }

    let target = cfg.target.trim();
    if target.is_empty() {
        errors.push(format!(
            "server {server_index}, route {route_index} (redirect): target is required"
        ));
        return;
    }

    if !(target.starts_with("http://") || target.starts_with("https://")) {
        errors.push(format!(
            "server {server_index}, route {route_index} (redirect): target must start with http:// or https://"
        ));
    }
}
