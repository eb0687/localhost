use crate::config::model::CgiConfig;
use crate::https::{self, Response, StatusCode, response_with_body};
use crate::router;

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
