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

    let health = warp::path("health").and(
        warp::path("liveness")
            .and(warp::get())
            .and_then(handler::health::liveness),
    );

    let signup = warp::path("signup")
        .and(warp::post())
        .and(with_app.clone())
        .and(with_auth.clone())
        .and(warp::body::json())
        .and_then(handler::signup);

    let me = warp::path("me").and(
        warp::get()
            .and(with_app.clone())
            .and(with_auth.clone())
            .and_then(handler::me)
            .or(warp::path("project").and(
                warp::path("list")
                    .and(warp::get())
                    .and(with_app.clone())
                    .and(with_auth.clone())
                    .and(warp::query())
                    .and_then(handler::me::project::list),
            )),
    );

    let project_form = warp::path("form").and(
        warp::path("get")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::project::form::get)
            .or(warp::path("list")
                .and(warp::get())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::query())
                .and_then(handler::project::form::list))
            .or(warp::path("answer").and(
                warp::post()
                    .and(with_app.clone())
                    .and(with_auth.clone())
                    .and(warp::body::json())
                    .and_then(handler::project::form::answer)
                    .or(warp::path("get")
                        .and(warp::get())
                        .and(with_app.clone())
                        .and(with_auth.clone())
                        .and(warp::query())
                        .and_then(handler::project::form::answer::get)),
            )),
    );

    let project = warp::path("project").and(
        warp::path("create")
            .and(warp::post())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::body::json())
            .and_then(handler::project::create)
            .or(warp::path("get")
                .and(warp::get())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::query())
                .and_then(handler::project::get))
            .or(warp::path("get-by-display-id")
                .and(warp::get())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::query())
                .and_then(handler::project::get_by_display_id))
            .or(warp::path("check-display-id")
                .and(warp::get())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::query())
                .and_then(handler::project::check_display_id))
            .or(warp::path("update")
                .and(warp::post())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::body::json())
                .and_then(handler::project::update))
            .or(warp::path("list")
                .and(warp::get())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::query())
                .and_then(handler::project::list))
            .or(warp::path("export")
                .and(warp::get())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::query())
                .and_then(handler::project::export))
            .or(project_form),
    );

    let form = warp::path("form").and(
        warp::path("get")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::form::get)
            .or(warp::path("list")
                .and(warp::get())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::query())
                .and_then(handler::form::list))
            .or(warp::path("create")
                .and(warp::post())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::body::json())
                .and_then(handler::form::create))
            .or(warp::path("answer").and(
                warp::path("list")
                    .and(warp::get())
                    .and(with_app.clone())
                    .and(with_auth.clone())
                    .and(warp::query())
                    .and_then(handler::form::answer::list)
                    .or(warp::path("export")
                        .and(warp::get())
                        .and(with_app.clone())
                        .and(with_auth.clone())
                        .and(warp::query())
                        .and_then(handler::form::answer::export)),
            )),
    );

    let form_answer = warp::path("form_answer").and(
        warp::path("get")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::form_answer::get),
    );

    let user = warp::path("user").and(
        warp::path("get")
            .and(warp::get())
            .and(with_app.clone())
            .and(with_auth.clone())
            .and(warp::query())
            .and_then(handler::user::get)
            .or(warp::path("list")
                .and(warp::get())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::query())
                .and_then(handler::user::list))
            .or(warp::path("export")
                .and(warp::get())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::query())
                .and_then(handler::user::export))
            .or(warp::path("update")
                .and(warp::post())
                .and(with_app.clone())
                .and(with_auth.clone())
                .and(warp::body::json())
                .and_then(handler::user::update)),
    );

    health
        .or(signup)
        .or(me)
        .or(project)
        .or(form)
        .or(form_answer)
        .or(user)
        .recover(handle_rejection)
        .with(warp::trace::request())
}
