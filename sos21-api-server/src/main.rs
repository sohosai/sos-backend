use std::net::SocketAddr;

use anyhow::{Context, Result};
use sos21_api_server::Config;
use structopt::StructOpt;
use tokio::runtime;
use tracing::{event, Level};
use url::Url;

#[derive(Debug, StructOpt)]
#[structopt(name = "sos21-api-server")]
struct Opt {
    #[structopt(short = "j", long, env = "SOS21_API_SERVER_THREADS")]
    threads: Option<usize>,
    #[structopt(long, env = "SOS21_API_SERVER_JWT_AUDIENCE")]
    jwt_audience: String,
    #[structopt(long, env = "SOS21_API_SERVER_JWT_ISSUER")]
    jwt_issuer: String,
    #[structopt(long, env = "SOS21_API_SERVER_JWT_KEYS_URL")]
    jwt_keys_url: Url,
    #[structopt(
        long,
        default_value = "5",
        env = "SOS21_API_SERVER_MAX_DATABASE_CONNECTIONS"
    )]
    max_database_connections: u32,
    #[structopt(short, long, env = "SOS21_API_SERVER_POSTGRES_URI")]
    postgres_uri: String,
    #[structopt(short, long, env = "SOS21_API_SERVER_BIND")]
    bind: SocketAddr,
}

fn main() {
    let opt = Opt::from_args();

    tracing_subscriber::fmt().pretty().init();

    if let Err(error) = run(opt) {
        event!(Level::ERROR, %error);
    }
}

fn run(opt: Opt) -> Result<()> {
    let mut builder = runtime::Builder::new_multi_thread();
    builder.enable_all();
    if let Some(j) = opt.threads {
        builder.worker_threads(j);
    }
    let runtime = builder
        .build()
        .context("Failed to build the Tokio Runtime")?;

    runtime.block_on(async move {
        let config = Config {
            jwt_audience: opt.jwt_audience,
            jwt_issuer: opt.jwt_issuer,
            jwt_keys_url: opt.jwt_keys_url,
            postgres_uri: opt.postgres_uri,
            max_database_connections: opt.max_database_connections,
        };
        let server = sos21_api_server::Server::new(config).await?;
        server.run(opt.bind).await;
        Ok(())
    })
}
