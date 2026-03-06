use crate::https::{Response, StatusCode, response_with_body};

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
