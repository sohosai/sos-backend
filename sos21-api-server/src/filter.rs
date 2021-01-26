use std::convert::Infallible;

use crate::app::App;

use warp::{reply::Reply, Filter};

mod authentication;
mod error;
mod handler;
mod login;

use authentication::authenticate;
pub use authentication::KeyStore;
use error::handle_rejection;
use handler::run_handler;
use login::login;
pub use error::model;

pub fn endpoints(
    app: App,
    key_store: KeyStore,
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    use crate::handler;

    let auth = authenticate(key_store, app);
    warp::path("health.liveness")
        .and(warp::get())
        .map(handler::health::liveness)
        .and_then(run_handler)
        .or(warp::path("signup")
            .and(warp::post())
            .and(auth.clone())
            .and(warp::body::json())
            .map(handler::signup)
            .and_then(run_handler))
        .or(warp::path("me")
            .and(warp::get())
            .and(auth.clone())
            .and_then(login)
            .map(handler::me)
            .and_then(run_handler))
        .or(warp::path("me.project.list")
            .and(warp::get())
            .and(auth.clone())
            .and_then(login)
            .and(warp::query())
            .map(handler::me::project::list)
            .and_then(run_handler))
        .or(warp::path("project.create")
            .and(warp::post())
            .and(auth.clone())
            .and_then(login)
            .and(warp::body::json())
            .map(handler::project::create)
            .and_then(run_handler))
        .or(warp::path("project.get")
            .and(warp::get())
            .and(auth.clone())
            .and_then(login)
            .and(warp::query())
            .map(handler::project::get)
            .and_then(run_handler))
        .or(warp::path("project.update")
            .and(warp::post())
            .and(auth.clone())
            .and_then(login)
            .and(warp::body::json())
            .map(handler::project::update)
            .and_then(run_handler))
        .or(warp::path("project.list")
            .and(warp::get())
            .and(auth.clone())
            .and_then(login)
            .and(warp::query())
            .map(handler::project::list)
            .and_then(run_handler))
        .or(warp::path("user.get")
            .and(warp::get())
            .and(auth.clone())
            .and_then(login)
            .and(warp::query())
            .map(handler::user::get)
            .and_then(run_handler))
        .or(warp::path("user.list")
            .and(warp::get())
            .and(auth.clone())
            .and_then(login)
            .and(warp::query())
            .map(handler::user::list)
            .and_then(run_handler))
        .or(warp::path("user.update")
            .and(warp::post())
            .and(auth)
            .and_then(login)
            .and(warp::body::json())
            .map(handler::user::update)
            .and_then(run_handler))
        .recover(handle_rejection)
        .with(warp::trace::request())
}
