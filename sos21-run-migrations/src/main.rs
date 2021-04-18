use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::{postgres::PgConnection, Connection};
use structopt::StructOpt;
use tokio::runtime;
use tracing::{event, Level};

#[derive(Debug, StructOpt)]
#[structopt(name = "sos21-run-migrations")]
struct Opt {
    #[structopt(short, long, env = "SOS21_RUN_MIGRATIONS_POSTGRES_URI")]
    postgres_uri: String,
    #[structopt(short, long)]
    wait: bool,
    #[structopt(
        short = "s",
        long,
        default_value = "1",
        env = "SOS21_RUN_MIGRATIONS_WAIT_RETRY_SEC"
    )]
    wait_retry_sec: u64,
    #[structopt(
        short = "c",
        long,
        default_value = "5",
        env = "SOS21_RUN_MIGRATIONS_WAIT_RETRY_COUNT"
    )]
    wait_retry_count: u64,
}

fn main() {
    let opt = Opt::from_args();

    tracing_subscriber::fmt().pretty().init();

    if let Err(error) = run(opt) {
        event!(Level::ERROR, ?error);
        std::process::exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Failed to build the Tokio Runtime")?;

    runtime.block_on(run_migrations(opt))
}

#[tracing::instrument]
async fn run_migrations(opt: Opt) -> Result<()> {
    let mut retry_count = opt.wait_retry_count;

    let mut conn = loop {
        match PgConnection::connect(&opt.postgres_uri).await {
            Ok(conn) => break conn,
            Err(sqlx::Error::Io(error)) if opt.wait && retry_count > 0 => {
                event!(Level::WARN, %error, retry_sec = %opt.wait_retry_sec, %retry_count);
                retry_count -= 1;
                tokio::time::sleep(Duration::from_secs(opt.wait_retry_sec)).await;
            }
            Err(err) => return Err(err).context("Failed to connect to the database"),
        }
    };

    sos21_database::migrate(&mut conn)
        .await
        .context("Failed to apply migrations")
}
