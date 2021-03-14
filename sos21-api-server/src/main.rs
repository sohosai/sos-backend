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
    #[structopt(long, env = "SOS21_API_SERVER_S3_ACCESS_KEY")]
    s3_access_key: String,
    #[structopt(long, env = "SOS21_API_SERVER_S3_ACCESS_SECRET")]
    s3_access_secret: String,
    #[structopt(long, env = "SOS21_API_SERVER_S3_REGION")]
    s3_region: String,
    #[structopt(long, env = "SOS21_API_SERVER_S3_ENDPOINT")]
    s3_endpoint: String,
    #[structopt(long, env = "SOS21_API_SERVER_S3_OBJECT_BUCKET")]
    s3_object_bucket: String,
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
            s3_access_key: opt.s3_access_key,
            s3_access_secret: opt.s3_access_secret,
            s3_region: opt.s3_region,
            s3_endpoint: opt.s3_endpoint,
            s3_object_bucket: opt.s3_object_bucket,
        };
        let server = sos21_api_server::Server::new(config).await?;
        server.run(opt.bind).await;
        Ok(())
    })
}
