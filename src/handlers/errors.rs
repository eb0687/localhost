use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::https::{Response, StatusCode, response_with_body};
use crate::utils::helpers::content_type_for_path;

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

pub fn error_response_with_pages(
    version: &str,
    status: StatusCode,
    error_pages: &HashMap<u16, String>,
) -> Response {
    let Some(path) = error_pages.get(&status.code()) else {
        return error_response(version, status);
    };

    let path = Path::new(path);
    match fs::read(path) {
        Ok(bytes) => response_with_body(
            version,
            status,
            content_type_for_path(path),
            bytes,
        ),
        Err(_) => error_response(version, status),
    }
}
