use std::net::SocketAddr;

use anyhow::{Context, Result};
use structopt::StructOpt;
use tokio::runtime;

#[derive(Debug, StructOpt)]
#[structopt(name = "sos21-api-server")]
struct Opt {
    #[structopt(short = "j", long, env = "SOS21_API_SERVER_THREADS")]
    threads: Option<usize>,
    #[structopt(short, long, env = "SOS21_API_SERVER_BIND")]
    bind: SocketAddr,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let mut builder = runtime::Builder::new_multi_thread();
    builder.enable_all();
    if let Some(j) = opt.threads {
        builder.worker_threads(j);
    }
    let runtime = builder
        .build()
        .context("Failed to build the Tokio Runtime")?;

    runtime.block_on(warp::serve(sos21_api_server::app()).run(opt.bind));

    Ok(())
}
