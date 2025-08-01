#![recursion_limit = "256"]

mod auth;
mod documents;
mod errors;
mod login;
mod server;
mod statics;

pub use crate::server::{ServeOptions, get_server_token, serve};
