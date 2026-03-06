pub mod cgi;
pub mod errors;
pub mod files;
pub mod redirect;
pub mod register;

pub use errors::error_response;
pub use register::register_routes;
