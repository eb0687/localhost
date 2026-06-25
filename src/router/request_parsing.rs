use std::collections::HashMap;

use crate::https::{HttpMethod, Request, StatusCode};

use super::Data;

pub(super) fn parse_host_header(header_bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(header_bytes).ok()?;
    let mut lines = text.split("\r\n");

    lines.next()?;

    for line in lines {
        if line.is_empty() {
            break;
        }

        let Some((name, value)) = line.split_once(':') else {
            continue;
        };

        if name.eq_ignore_ascii_case("host") {
            return Some(value.trim().to_string());
        }
    }

    None
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn percent_decode_path(raw: &str) -> Result<String, String> {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'%' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }

        if i + 2 >= bytes.len() {
            return Err(
                "incomplete percent encoding in request path".to_string()
            );
        }

        let hi = hex_value(bytes[i + 1]).ok_or_else(|| {
            "invalid percent encoding in request path".to_string()
        })?;
        let lo = hex_value(bytes[i + 2]).ok_or_else(|| {
            "invalid percent encoding in request path".to_string()
        })?;

        out.push((hi << 4) | lo);
        i += 3;
    }

    String::from_utf8(out)
        .map_err(|_| "request path is not valid UTF-8".to_string())
}

fn path_contains_forbidden_segments(path: &str) -> bool {
    path.split('/').any(|segment| segment == "..")
}

pub(super) fn parse_request(
    header_bytes: &[u8],
    body: &[u8],
) -> Result<Request, (StatusCode, String)> {
    let bad_request =
        |reason: &str| (StatusCode::BadRequest, reason.to_string());
    let text = std::str::from_utf8(header_bytes)
        .map_err(|_| bad_request("request headers are not valid UTF-8"))?;
    let mut lines = text.split("\r\n");

    let request_line = lines
        .next()
        .ok_or_else(|| bad_request("missing request line"))?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| bad_request("missing HTTP method"))?;
    let raw_path = parts
        .next()
        .ok_or_else(|| bad_request("missing request path"))?;
    let version = parts
        .next()
        .ok_or_else(|| bad_request("missing HTTP version"))?;

    if parts.next().is_some() {
        return Err(bad_request("request line has extra fields"));
    }

    if version != "HTTP/1.1" && version != "HTTP/1.0" {
        return Err((
            StatusCode::VersionNotSupported,
            "unsupported HTTP version".to_string(),
        ));
    }

    let method = HttpMethod::from_str(method);
    if matches!(method, HttpMethod::Post) && body.is_empty() {
        return Err(bad_request("POST request requires a non-empty body"));
    }

    let mut headers = crate::https::HeaderMap::default();
    for line in lines {
        if line.is_empty() {
            break;
        }

        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name, value);
        }
    }

    let (raw_path_only, query) = raw_path
        .split_once('?')
        .map(|(p, q)| (p, q.to_string()))
        .unwrap_or((raw_path, String::new()));

    let path = percent_decode_path(raw_path_only)
        .map_err(|reason| (StatusCode::BadRequest, reason))?;

    if !path.starts_with('/') {
        return Err(bad_request("request path must start with '/'"));
    }

    if path_contains_forbidden_segments(&path) {
        return Err((
            StatusCode::Forbidden,
            "request path contains forbidden traversal segment".to_string(),
        ));
    }

    Ok(Request {
        method,
        path,
        query,
        version: version.to_string(),
        headers,
        data: Data {
            body: body.to_vec(),
            path_value: HashMap::new(),
            query_value: HashMap::new(),
            session_id: None,
            is_new_session: false,
            error_pages: HashMap::new(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_request, percent_decode_path};
    use crate::https::StatusCode;

    #[test]
    fn percent_decodes_path_spaces() {
        assert_eq!(
            percent_decode_path("/upload/hello%20world.txt").unwrap(),
            "/upload/hello world.txt"
        );
    }

    #[test]
    fn rejects_invalid_percent_encoding() {
        let req = b"GET /upload/%ZZ.txt HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let err = parse_request(req, b"").expect_err("request should fail");

        assert_eq!(err.0.code(), StatusCode::BadRequest.code());
    }

    #[test]
    fn rejects_decoded_dot_dot_segments() {
        let req =
            b"GET /upload/%2e%2e/bad.txt HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let err = parse_request(req, b"").expect_err("request should fail");

        assert_eq!(err.0.code(), StatusCode::Forbidden.code());
    }
}
