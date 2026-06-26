use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::model::CgiConfig;
use crate::https::{self, HeaderMap, Response, StatusCode, response_with_body};
use crate::router;

use super::errors::error_response_with_pages;

pub fn cgi_factory(
    cgi_config: CgiConfig,
) -> impl Fn(&https::Request, &router::Data) -> Response + Send + Sync {
    move |req: &https::Request, data: &router::Data| -> Response {
        handle_cgi(req, data, &cgi_config)
    }
}

fn handle_cgi(
    req: &https::Request,
    data: &router::Data,
    cfg: &CgiConfig,
) -> Response {
    let root = Path::new(&cfg.root);

    if !root.is_dir() {
        return error_response_with_pages(
            &req.version,
            StatusCode::InternalServerError,
            &data.error_pages,
        );
    }

    let Some(rel_script_path) = route_relative_subpath(&cfg.path, &req.path)
    else {
        return error_response_with_pages(
            &req.version,
            StatusCode::NotFound,
            &data.error_pages,
        );
    };

    if rel_script_path.is_empty() {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    }

    let Some(script_path) = safe_join_under_root(root, rel_script_path) else {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    };

    if !script_path.is_file() {
        return error_response_with_pages(
            &req.version,
            StatusCode::NotFound,
            &data.error_pages,
        );
    }

    let has_expected_extension = script_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{ext}") == cfg.extension)
        .unwrap_or(false);

    if !has_expected_extension {
        return error_response_with_pages(
            &req.version,
            StatusCode::Forbidden,
            &data.error_pages,
        );
    }

    let output = match run_cgi(req, data, cfg, &script_path, rel_script_path) {
        Ok(output) => output,
        Err(_) => {
            return error_response_with_pages(
                &req.version,
                StatusCode::InternalServerError,
                &data.error_pages,
            );
        }
    };

    parse_cgi_output(&req.version, output)
}

fn run_cgi(
    req: &https::Request,
    data: &router::Data,
    cfg: &CgiConfig,
    script_path: &Path,
    path_info: &str,
) -> Result<Vec<u8>, String> {
    let script_path = script_path
        .canonicalize()
        .map_err(|e| format!("failed to canonicalize CGI script path: {e}"))?;
    let mut child = Command::new(&cfg.interpreter)
        .arg(script_path)
        .current_dir(Path::new(&cfg.root))
        .env("REQUEST_METHOD", method_name(&req.method))
        .env("PATH_INFO", format!("/{path_info}"))
        .env("QUERY_STRING", &req.query)
        .env("CONTENT_LENGTH", data.body.len().to_string())
        .env(
            "CONTENT_TYPE",
            req.headers.get("content-type").unwrap_or(""),
        )
        .env("SCRIPT_NAME", &req.path)
        .env("SERVER_PROTOCOL", &req.version)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn CGI: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&data.body)
            .map_err(|e| format!("failed to write CGI stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("failed to read CGI output: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "CGI exited with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(output.stdout)
}

fn parse_cgi_output(version: &str, output: Vec<u8>) -> Response {
    if let Some((header_bytes, body)) = split_cgi_headers(&output) {
        let mut headers = HeaderMap::default();
        let header_text = String::from_utf8_lossy(header_bytes);
        let mut content_type = "text/plain; charset=utf-8".to_string();

        for line in header_text.lines() {
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };

            if name.eq_ignore_ascii_case("content-type") {
                content_type = value.trim().to_string();
            } else {
                headers.insert(name, value);
            }
        }

        headers.insert("Content-Type", &content_type);
        headers.insert("Content-Length", &body.len().to_string());
        headers.insert("Connection", "close");

        return Response {
            version: version.to_string(),
            status: StatusCode::Ok,
            headers,
            body: body.to_vec(),
        };
    }

    response_with_body(
        version,
        StatusCode::Ok,
        "text/plain; charset=utf-8",
        output,
    )
}

fn split_cgi_headers(output: &[u8]) -> Option<(&[u8], &[u8])> {
    if let Some(pos) = output.windows(4).position(|w| w == b"\r\n\r\n") {
        return Some((&output[..pos], &output[pos + 4..]));
    }

    if let Some(pos) = output.windows(2).position(|w| w == b"\n\n") {
        return Some((&output[..pos], &output[pos + 2..]));
    }

    None
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

fn method_name(method: &https::HttpMethod) -> &'static str {
    match method {
        https::HttpMethod::Get => "GET",
        https::HttpMethod::Post => "POST",
        https::HttpMethod::Delete => "DELETE",
        https::HttpMethod::Unknown(_) => "UNKNOWN",
    }
}
