mod documents;
mod errors;
mod login;
mod server;
mod statics;

pub use crate::server::{get_access_token, serve, ServeOptions};
