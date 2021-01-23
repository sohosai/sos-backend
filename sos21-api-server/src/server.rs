use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;

use crate::app::App;
use crate::config::Config;
use crate::filter::{self, KeyStore};

use anyhow::Result;
use tokio::{task::JoinHandle, time};
use tracing::{event, Level};

#[derive(Debug)]
pub struct Server {
    app: App,
    key_store: KeyStore,
    key_refresh_worker: JoinHandle<Infallible>,
}

const FETCH_MINIMUM_INTERVAL: u64 = 5 * 60 * 60;

fn spawn_key_refresh_worker(key_store: KeyStore) -> JoinHandle<Infallible> {
    tokio::spawn(async move {
        loop {
            let interval_sec = match key_store.refresh().await {
                Ok(max_age) => {
                    let interval = max_age.unwrap_or(0).max(FETCH_MINIMUM_INTERVAL);
                    event!(Level::INFO, ?max_age, interval, "Refreshed the JWT keys",);
                    interval
                }
                Err(error) => {
                    event!(
                        Level::ERROR,
                        %error,
                        retry = FETCH_MINIMUM_INTERVAL,
                        "Failed to refresh JWT keys",
                    );
                    FETCH_MINIMUM_INTERVAL
                }
            };
            assert!(interval_sec >= FETCH_MINIMUM_INTERVAL);
            time::sleep(Duration::from_secs(interval_sec)).await;
        }
    })
}

impl Server {
    pub async fn new(config: Config) -> Result<Self> {
        let app = App::new(config.clone()).await?;
        let key_store = KeyStore::new(config.jwt_keys_url.clone());

        // fill KeyStore here (i.e. not in refresh worker)
        // to ensure it is filled at `run` and to catch early errors
        key_store.refresh().await?;
        let key_refresh_worker = spawn_key_refresh_worker(key_store.clone());

        Ok(Server {
            app,
            key_store,
            key_refresh_worker,
        })
    }

    pub async fn run(&self, bind: impl Into<SocketAddr>) {
        warp::serve(filter::endpoints(self.app.clone(), self.key_store.clone()))
            .run(bind)
            .await
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.key_refresh_worker.abort();
    }
}
