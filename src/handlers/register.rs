use std::path;

use crate::config::{AppConfig, model::RouteRule, parse::parse_route_key};
use crate::https;
use crate::router::Router;

use super::cgi::cgi_factory;
use super::files::{dir_server_factory, file_server_factory};
use super::redirect::redirect_factory;

pub fn register_routes(
    app_config: &AppConfig,
    router: &mut Router,
) -> Result<(), String> {
    for (path_key, route) in &app_config.config.routes {
        let route_key = parse_route_key(path_key)
            .map_err(|e| format!("invalid route key '{}': {}", path_key, e))?;

        let port = route_key.port;
        let pattern = route_key.path;

        match route {
            RouteRule::FileServer(file_server_config) => {
                let methods = file_server_config
                    .allowed_verbs
                    .clone()
                    .unwrap_or_else(|| vec![https::HttpMethod::Get]);

                if methods.is_empty() {
                    return Err(format!(
                        "route '{}' has empty allowed_verbs",
                        pattern
                    ));
                }

                let mut file_server_cfg = file_server_config.clone();
                file_server_cfg.mount_path = pattern.clone();

                let root_path = path::Path::new(&file_server_cfg.root);
                if root_path.is_dir() {
                    let dir_pattern = if pattern == "/" {
                        "/*rest".to_string()
                    } else {
                        format!("{pattern}/*rest")
                    };
                    router.add_route(
                        port,
                        &dir_pattern,
                        methods,
                        dir_server_factory(file_server_cfg),
                    );
                } else if root_path.is_file() {
                    router.add_route(
                        port,
                        &pattern,
                        methods,
                        file_server_factory(file_server_cfg),
                    );
                } else {
                    return Err(format!(
                        "route '{}' file_server root '{}' does not exist",
                        pattern, file_server_cfg.root
                    ));
                }
            }
            RouteRule::Cgi(cgi_config) => {
                router.add_route(
                    port,
                    &pattern,
                    vec![https::HttpMethod::Get, https::HttpMethod::Post],
                    cgi_factory(cgi_config.clone()),
                );
            }
            RouteRule::Redirect(redirect_config) => {
                router.add_route(
                    port,
                    &pattern,
                    vec![https::HttpMethod::Get],
                    redirect_factory(redirect_config.clone()),
                );
            }
        }
    }

    Ok(())
}
