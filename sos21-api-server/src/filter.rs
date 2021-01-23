use std::convert::Infallible;

use crate::app::App;

use warp::{reply::Reply, Filter};

mod authentication;
mod error;
mod handler;

use authentication::authenticate;
pub use authentication::KeyStore;
use error::handle_rejection;
use handler::run_handler;

pub fn endpoints(
    app: App,
    key_store: KeyStore,
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    use crate::handler;

    let auth = authenticate(key_store, app);
    warp::path("health")
        .and(
            warp::path("liveness")
                .and(warp::get())
                .map(handler::health::liveness)
                .and_then(run_handler),
        )
        .or(warp::path("signup")
            .and(warp::post())
            .and(auth.clone())
            .and(warp::body::json())
            .map(handler::signup)
            .and_then(run_handler))
        .or(warp::path("me")
            .and(warp::get())
            .and(auth.clone())
            .map(handler::me)
            .and_then(run_handler))
        .recover(handle_rejection)
        .with(warp::trace::request())
}
