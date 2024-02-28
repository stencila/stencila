mod documents;
mod errors;
mod secrets;
mod server;
mod statics;

pub use crate::server::{serve, ServeOptions};
