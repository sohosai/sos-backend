mod app;
mod config;
mod server;

pub mod filter;
pub mod handler;

pub use config::Config;
pub use server::Server;

mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
