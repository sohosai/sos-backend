use std::convert::Infallible;

use crate::app::App;

use warp::{reply::Reply, Filter};

mod authentication;
mod error;

use authentication::authenticate;
pub use authentication::{AuthenticationInfo, KeyStore};
use error::handle_rejection;
pub use error::model;

pub fn endpoints(
    app: App,
    key_store: KeyStore,
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    use crate::handler;

    let with_auth = authenticate(key_store, app.config().clone());
    let with_app = warp::any().map(move || app.clone());

    warp::path("health.liveness")
        .and(warp::get())
        .and_then(handler::health::liveness)
        .or(warp::path("signup")
            .and(warp::post())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::body::json())
            .and_then(handler::signup))
        .or(warp::path("me")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and_then(handler::me))
        .or(warp::path("me.project.list")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::me::project::list))
        .or(warp::path("project.create")
            .and(warp::post())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::body::json())
            .and_then(handler::project::create))
        .or(warp::path("project.get")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::project::get))
        .or(warp::path("project.get-by-display-id")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::project::get_by_display_id))
        .or(warp::path("project.update")
            .and(warp::post())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::body::json())
            .and_then(handler::project::update))
        .or(warp::path("project.list")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::project::list))
        .or(warp::path("project.export")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::project::export))
        .or(warp::path("user.get")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::user::get))
        .or(warp::path("user.list")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::user::list))
        .or(warp::path("user.export")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::user::export))
        .or(warp::path("user.update")
            .and(warp::post())
            .and(with_app)
            .and(with_auth)
            .and(warp::body::json())
            .and_then(handler::user::update))
        .recover(handle_rejection)
        .with(warp::trace::request())
}
