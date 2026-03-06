use crate::config::model::RedirectConfig;
use crate::https::{self, Response, StatusCode, response_with_body};
use crate::router;

pub fn redirect_factory(
    redirect_config: RedirectConfig,
) -> impl Fn(&https::Request, &router::Data) -> Response + Send + Sync {
    move |req: &https::Request, _data: &router::Data| -> Response {
        let mut resp = response_with_body(
            &req.version,
            StatusCode::Found,
            "text/plain; charset=utf-8",
            format!("Redirecting to {}", redirect_config.target).into_bytes(),
        );
        resp.headers.insert("Location", &redirect_config.target);
        resp
    }
}
