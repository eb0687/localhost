use std::collections::HashMap;
use std::path;
use std::sync::Arc;

use crate::config::{
    AppConfig,
    model::{FileServerConfig, RouteRule, ServerConfig},
};
use crate::https;
use crate::router::{Route, Router, VirtualServer};

use super::cgi::cgi_factory;
use super::files::{dir_server_factory, file_server_factory};
use super::redirect::redirect_factory;

pub fn register_routes(
    app_config: &AppConfig,
    router: &mut Router,
) -> Result<(), String> {
    let mut registered = 0;

    for (server_index, server_config) in
        app_config.config.servers.iter().enumerate()
    {
        match build_virtual_server(server_config) {
            Ok(virtual_server) => {
                router.add_virtual_server(virtual_server);
                registered += 1;
            }
            Err(err) => {
                eprintln!("skipping server {server_index}: {err}");
            }
        }
    }

    if registered == 0 {
        return Err("no valid virtual servers registered".to_string());
    }

    Ok(())
}

fn build_virtual_server(
    server_config: &ServerConfig,
) -> Result<VirtualServer, String> {
    let mut virtual_server = VirtualServer {
        ports: server_config.ports.clone(),
        server_names: server_config
            .server_name
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect(),
        client_max_body_size: server_config.client_max_body_size,
        error_pages: parse_error_pages(server_config.error_pages.clone())?,
        routes: Vec::new(),
    };

    for route in &server_config.routes {
        match route {
            RouteRule::FileServer(file_server_config) => {
                register_file_server_route(
                    &mut virtual_server,
                    &file_server_config,
                )?;
            }
            RouteRule::Cgi(cgi_config) => {
                let pattern = if cgi_config.path == "/" {
                    "/*rest".to_string()
                } else {
                    format!("{}/*rest", cgi_config.path)
                };

                virtual_server.routes.push(Route {
                    methods: vec![
                        https::HttpMethod::Get,
                        https::HttpMethod::Post,
                    ],
                    pattern,
                    handler: Arc::new(cgi_factory(cgi_config.clone())),
                });
            }
            RouteRule::Redirect(redirect_config) => {
                virtual_server.routes.push(Route {
                    methods: vec![https::HttpMethod::Get],
                    pattern: redirect_config.path.clone(),
                    handler: Arc::new(redirect_factory(
                        redirect_config.clone(),
                    )),
                });
            }
        }
    }

    virtual_server.routes.sort_by(|a, b| {
        let a_is_catch_all = a.pattern.contains('*');
        let b_is_catch_all = b.pattern.contains('*');

        a_is_catch_all
            .cmp(&b_is_catch_all)
            .then_with(|| b.pattern.len().cmp(&a.pattern.len()))
    });

    Ok(virtual_server)
}

fn register_file_server_route(
    virtual_server: &mut VirtualServer,
    file_server_config: &FileServerConfig,
) -> Result<(), String> {
    let pattern = file_server_config.path.clone();

    let methods = file_server_config
        .allowed_methods
        .clone()
        .unwrap_or_else(|| vec![https::HttpMethod::Get]);

    if methods.is_empty() {
        return Err(format!("route '{}' has empty allowed_methods", pattern));
    }

    let mut cfg = file_server_config.clone();
    cfg.mount_path = pattern.clone();

    let root_path = path::Path::new(&cfg.root);
    if root_path.is_dir() {
        let dir_pattern = if pattern == "/" {
            "/*rest".to_string()
        } else {
            format!("{pattern}/*rest")
        };

        virtual_server.routes.push(Route {
            methods,
            pattern: dir_pattern,
            handler: Arc::new(dir_server_factory(cfg)),
        });
    } else if root_path.is_file() {
        virtual_server.routes.push(Route {
            methods,
            pattern: pattern.clone(),
            handler: Arc::new(file_server_factory(cfg)),
        });
    } else {
        return Err(format!(
            "route '{}' file_server root '{}' does not exist",
            pattern, cfg.root
        ));
    }

    Ok(())
}

fn parse_error_pages(
    raw: Option<HashMap<String, String>>,
) -> Result<HashMap<u16, String>, String> {
    let mut parsed = HashMap::new();

    let Some(raw) = raw else {
        return Ok(parsed);
    };

    for (code_str, path) in raw {
        let code = code_str.parse::<u16>().map_err(|_| {
            format!("error page code '{code_str}' is not a valid status code")
        })?;

        parsed.insert(code, path);
    }

    Ok(parsed)
}
