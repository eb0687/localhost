use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::config::model::FileServerConfig;
use crate::https::{self, Response, StatusCode, response_with_body};
use crate::router;
use crate::utils::helpers::content_type_for_path;

use super::errors::error_response_with_pages;

pub fn dir_server_factory(
    cfg: FileServerConfig,
) -> impl Fn(&https::Request, &router::Data) -> Response + Send + Sync {
    move |req: &https::Request, data: &router::Data| -> Response {
        match req.method {
            https::HttpMethod::Get => handle_get_dir(req, data, &cfg),
            https::HttpMethod::Post => handle_upload(req, data, &cfg),
            https::HttpMethod::Delete => handle_delete(req, data, &cfg),
            _ => error_response_with_pages(
                &req.version,
                StatusCode::MethodNotAllowed,
                &data.error_pages,
            ),
        }
    }
}

pub fn file_server_factory(
    cfg: FileServerConfig,
) -> impl Fn(&https::Request, &router::Data) -> Response + Send + Sync {
    move |req: &https::Request, data: &router::Data| -> Response {
        match req.method {
            https::HttpMethod::Get => handle_get_file(req, data, &cfg),
            https::HttpMethod::Post => handle_upload(req, data, &cfg),
            https::HttpMethod::Delete => handle_delete(req, data, &cfg),
            _ => error_response_with_pages(
                &req.version,
                StatusCode::MethodNotAllowed,
                &data.error_pages,
            ),
        }
    }
}

fn handle_get_dir(
    req: &https::Request,
    data: &router::Data,
    cfg: &FileServerConfig,
) -> Response {
    let root = Path::new(&cfg.root);
    let Some(target) = route_target(root, &cfg.mount_path, &req.path) else {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    };

    let meta = match fs::metadata(&target) {
        Ok(m) => m,
        Err(_) => {
            return error_response_with_pages(
                &req.version,
                StatusCode::NotFound,
                &data.error_pages,
            );
        }
    };

    if meta.is_file() {
        return match fs::read(&target) {
            Ok(bytes) => response_with_body(
                &req.version,
                StatusCode::Ok,
                content_type_for_path(&target),
                bytes,
            ),
            Err(_) => error_response_with_pages(
                &req.version,
                StatusCode::InternalServerError,
                &data.error_pages,
            ),
        };
    }

    if !meta.is_dir() {
        return error_response_with_pages(
            &req.version,
            StatusCode::NotFound,
            &data.error_pages,
        );
    }

    let default_file = cfg.default_file.as_deref().unwrap_or("index.html");
    let index_path = target.join(default_file);
    if index_path.is_file() {
        return match fs::read(&index_path) {
            Ok(bytes) => response_with_body(
                &req.version,
                StatusCode::Ok,
                content_type_for_path(&index_path),
                bytes,
            ),
            Err(_) => error_response_with_pages(
                &req.version,
                StatusCode::InternalServerError,
                &data.error_pages,
            ),
        };
    }

    if !cfg.directory_listing.unwrap_or(false) {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    }

    let entries = match fs::read_dir(&target) {
        Ok(rd) => rd,
        Err(_) => {
            return error_response_with_pages(
                &req.version,
                StatusCode::InternalServerError,
                &data.error_pages,
            );
        }
    };

    let mut items: Vec<String> = entries
        .flatten()
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.path().is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect();

    items.sort();

    let mut body = String::from("<html><body><h1>Directory listing</h1><ul>");
    let base = if req.path.ends_with('/') {
        req.path.clone()
    } else {
        format!("{}/", req.path)
    };

    for item in items {
        body.push_str("<li><a href=\"");
        body.push_str(&base);
        body.push_str(&item);
        body.push_str("\">");
        body.push_str(&item);
        body.push_str("</a></li>");
    }

    body.push_str("</ul></body></html>");

    response_with_body(
        &req.version,
        StatusCode::Ok,
        "text/html; charset=utf-8",
        body.into_bytes(),
    )
}

fn handle_get_file(
    req: &https::Request,
    data: &router::Data,
    cfg: &FileServerConfig,
) -> Response {
    let root = Path::new(&cfg.root);

    if root.is_file() {
        if req.path != cfg.mount_path {
            return error_response_with_pages(
                &req.version,
                StatusCode::NotFound,
                &data.error_pages,
            );
        }

        return match fs::read(root) {
            Ok(bytes) => response_with_body(
                &req.version,
                StatusCode::Ok,
                content_type_for_path(root),
                bytes,
            ),
            Err(_) => error_response_with_pages(
                &req.version,
                StatusCode::NotFound,
                &data.error_pages,
            ),
        };
    }

    let Some(target) = route_target(root, &cfg.mount_path, &req.path) else {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    };

    if !target.is_file() {
        return error_response_with_pages(
            &req.version,
            StatusCode::NotFound,
            &data.error_pages,
        );
    }

    match fs::read(&target) {
        Ok(bytes) => response_with_body(
            &req.version,
            StatusCode::Ok,
            content_type_for_path(&target),
            bytes,
        ),
        Err(_) => error_response_with_pages(
            &req.version,
            StatusCode::NotFound,
            &data.error_pages,
        ),
    }
}

fn handle_upload(
    req: &https::Request,
    data: &router::Data,
    cfg: &FileServerConfig,
) -> Response {
    let root = Path::new(&cfg.root);

    if !root.is_dir() {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    }

    let Some(rel_path) = route_relative_subpath(&cfg.mount_path, &req.path)
    else {
        return error_response_with_pages(
            &req.version,
            StatusCode::NotFound,
            &data.error_pages,
        );
    };

    if rel_path.is_empty() {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    }

    let Some(target) = safe_join_under_root(root, rel_path) else {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    };

    if target.is_dir() {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    }

    if let Some(parent) = target.parent() {
        if !parent.starts_with(root) {
            return error_response_with_pages(
                &req.version,
                StatusCode::Forbidden,
                &data.error_pages,
            );
        }

        if let Err(_) = fs::create_dir_all(parent) {
            return error_response_with_pages(
                &req.version,
                StatusCode::InternalServerError,
                &data.error_pages,
            );
        }
    }

    match fs::write(&target, &data.body) {
        Ok(_) => response_with_body(
            &req.version,
            StatusCode::Created,
            "text/plain; charset=utf-8",
            b"created\n".to_vec(),
        ),
        Err(_) => error_response_with_pages(
            &req.version,
            StatusCode::InternalServerError,
            &data.error_pages,
        ),
    }
}

fn handle_delete(
    req: &https::Request,
    data: &router::Data,
    cfg: &FileServerConfig,
) -> Response {
    let root = Path::new(&cfg.root);

    if !root.is_dir() {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    }

    let Some(target) = route_target(root, &cfg.mount_path, &req.path) else {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    };

    if target.is_dir() {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    }

    if !target.exists() {
        return error_response_with_pages(
            &req.version,
            StatusCode::NotFound,
            &data.error_pages,
        );
    }

    match fs::remove_file(&target) {
        Ok(_) => response_with_body(
            &req.version,
            StatusCode::NoContent,
            "text/plain",
            Vec::new(),
        ),
        Err(_) => error_response_with_pages(
            &req.version,
            StatusCode::InternalServerError,
            &data.error_pages,
        ),
    }
}

fn route_target(
    root: &Path,
    mount_path: &str,
    req_path: &str,
) -> Option<PathBuf> {
    let rel_path = route_relative_subpath(mount_path, req_path)?;
    safe_join_under_root(root, rel_path)
}

fn route_relative_subpath<'a>(
    mount_path: &str,
    req_path: &'a str,
) -> Option<&'a str> {
    if mount_path == "/" {
        return Some(req_path.trim_start_matches('/'));
    }

    if req_path == mount_path {
        return Some("");
    }

    let prefix = format!("{mount_path}/");
    req_path.strip_prefix(&prefix)
}

fn safe_join_under_root(root: &Path, rel: &str) -> Option<PathBuf> {
    let mut out = root.to_path_buf();

    for comp in Path::new(rel).components() {
        match comp {
            Component::Normal(seg) => out.push(seg),
            Component::CurDir => {}
            Component::RootDir => {}
            Component::ParentDir | Component::Prefix(_) => return None,
        }
    }

    Some(out)
}
