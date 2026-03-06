use std::{
    fs,
    path::{self, Path},
};

use crate::{
    config::{
        AppConfig,
        model::{CgiConfig, FileServerConfig, RedirectConfig, RouteRule},
        parse::parse_route_key,
    },
    https::{self, Response, StatusCode, response_with_body},
    router::{self, Router},
};

pub fn error_response(version: &str, status: StatusCode) -> Response {
    let reason = status.reason();
    let body = format!(
        "<html><body><h1>{} {}</h1></body></html>",
        status.code(),
        reason
    )
    .into_bytes();
    response_with_body(version, status, "text/html; charset=utf-8", body)
}

pub fn register_routes(
    app_config: &AppConfig,
    router: &mut Router,
) -> Result<(), String> {
    for (path, route) in app_config.config.routes.iter() {
        let route_key = parse_route_key(path)
            .map_err(|e| format!("invalid route key '{}': {}", path, e))?;

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

                let root_path = path::Path::new(&file_server_config.root);
                if root_path.is_dir() {
                    router.add_route(
                        port,
                        &pattern,
                        methods,
                        dir_server_factory(file_server_config.clone()),
                    );
                } else if root_path.is_file() {
                    router.add_route(
                        port,
                        &pattern,
                        methods,
                        file_server_factory(file_server_config.clone()),
                    );
                } else {
                    return Err(format!(
                        "route '{}' file_server root '{}' does not exist",
                        pattern, file_server_config.root
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

pub fn dir_server_factory(
    cfg: FileServerConfig,
) -> impl Fn(&https::Request, &router::Data) -> Response + Send + Sync {
    move |req: &https::Request, _data: &router::Data| -> Response {
        if req.method != https::HttpMethod::Get {
            return error_response(&req.version, StatusCode::MethodNotAllowed);
        }

        let root = Path::new(&cfg.root);

        let meta = match fs::metadata(root) {
            Ok(m) => m,
            Err(_) => {
                return error_response(&req.version, StatusCode::NotFound);
            }
        };

        if !meta.is_dir() {
            return error_response(&req.version, StatusCode::NotFound);
        };

        let index_path = root.join("index.html");
        if index_path.is_file() {
            return match fs::read(&index_path) {
                Ok(bytes) => response_with_body(
                    &req.version,
                    StatusCode::Ok,
                    "text/html; charset=utf-8",
                    bytes,
                ),
                Err(_) => error_response(
                    &req.version,
                    StatusCode::InternalServerError,
                ),
            };
        }

        let listing_enabled = cfg.directory_listing.unwrap_or(false);
        if !listing_enabled {
            return error_response(&req.version, StatusCode::Forbidden);
        };

        let mut items: Vec<String> = Vec::new();
        let entries = match fs::read_dir(root) {
            Ok(rd) => rd,
            Err(_) => {
                return error_response(
                    &req.version,
                    StatusCode::InternalServerError,
                );
            }
        };

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.path().is_dir();
            if is_dir {
                items.push(format!("{}/", name));
            } else {
                items.push(name);
            }
        }

        items.sort();

        let mut body =
            String::from("<html><body><h1>Directory listing</h1><ul>");
        for item in items {
            body.push_str("<li>");
            body.push_str(&item);
            body.push_str("</li>");
        }

        body.push_str("</ul></body></html>");

        response_with_body(
            &req.version,
            StatusCode::Ok,
            "text/html; charset=utf-8",
            body.into_bytes(),
        )
    }
}

pub fn file_server_factory(
    cfg: FileServerConfig,
) -> impl Fn(&https::Request, &router::Data) -> Response + Send + Sync {
    move |req: &https::Request, _data: &router::Data| -> Response {
        if req.method != https::HttpMethod::Get {
            return error_response(&req.version, StatusCode::MethodNotAllowed);
        }

        match fs::read(cfg.root.as_str()) {
            Ok(bytes) => response_with_body(
                &req.version,
                StatusCode::Ok,
                "application/octet-stream",
                bytes,
            ),
            Err(_) => error_response(&req.version, StatusCode::NotFound),
        }
    }
}

pub fn cgi_factory(
    cgi_config: CgiConfig,
) -> impl Fn(&https::Request, &router::Data) -> Response + Send + Sync {
    move |req: &https::Request, _data: &router::Data| -> Response {
        response_with_body(
            &req.version,
            StatusCode::InternalServerError,
            "text/plain; charset=utf-8",
            format!("cgi not implemented yet: {}", cgi_config.root)
                .into_bytes(),
        )
    }
}

pub fn redirect_factory(
    redirect_config: RedirectConfig,
) -> impl Fn(&https::Request, &router::Data) -> Response + Send + Sync {
    move |req: &https::Request, _data: &router::Data| -> Response {
        let mut resp = response_with_body(
            &req.version,
            StatusCode::NoContent,
            "text/plain; charset=utf-8",
            format!("Redirecting to {}", redirect_config.target).into_bytes(),
        );
        resp.headers.insert("Location", &redirect_config.target);
        resp
    }
}
