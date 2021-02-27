use std::convert::Infallible;

use crate::app::App;

use warp::{reply::Reply, Filter};

mod authentication;
mod error;

use authentication::authenticate;
pub use authentication::{AuthenticationInfo, KeyStore};
use error::handle_rejection;
pub use error::model;

macro_rules! route {
    (@method GET) => { warp::get().and(warp::query()) };
    (@method POST) => { warp::post().and(warp::body::json()) };
    (@path) => { warp::any() };
    (@path $name:literal) => { warp::path($name) };
    (@options $with_auth:ident, $with_app:ident, {noauth}) => { warp::any() };
    (@options $with_auth:ident, $with_app:ident, {}) => { $with_app.clone().and($with_auth.clone()) };
    ($with_auth:ident, $with_app:ident, / $name:literal { $($inner:tt)+ }) => {
        warp::path($name)
            .and(routes!{ $with_auth, $with_app, $($inner)+ })
    };
    ($with_auth:ident, $with_app:ident,
       / $name_1:literal $(/ $name_n:literal)+ => {$($options:tt),*} $method:ident ($handler:path)
    ) => {
        warp::path($name_1)
            .and(route!{ $with_auth, $with_app, $(/ $name_n)+ => {$($options),*} $method ($handler) })
    };
    ($with_auth:ident, $with_app:ident,
       / $($name:literal)? => {$($options:tt),*} $method:ident ($handler:path)
    ) => {
        route!(@path $($name)?)
            .and(warp::path::end()
                .and(route!(@options $with_auth, $with_app, {$($options),*}))
                .and(route!(@method $method))
                .and_then($handler))
    };
}

macro_rules! routes {
    ($with_auth:ident, $with_app:ident,
       / $($name_1:literal)/ * $({ $($inner_1:tt)+ })? $(=> $({$($option_1:tt),*})? $method_1:ident ($handler_1:path) )?
         $(, / $($name_n:literal)/ * $({ $($inner_n:tt)+ })? $(=> $({$($option_n:tt),*})? $method_n:ident ($handler_n:path) )? )*
         $(,)?
    ) => {
        route!{ $with_auth, $with_app, / $($name_1)/ * $({ $($inner_1)+ })? $(=> {$($($option_1),*)?} $method_1 ($handler_1))? }
            $( .or( route!{ $with_auth, $with_app, / $($name_n)/ * $({ $($inner_n)+ })? $(=> {$($($option_n),*)?} $method_n ($handler_n))? } ) )*
    }
}

pub fn endpoints(
    app: App,
    key_store: KeyStore,
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    use crate::handler;

    let with_auth = authenticate(key_store, app.config().clone());
    let with_app = warp::any().map(move || app.clone());

    let routes = routes! { with_auth, with_app,
        / "health" / "liveness" => {noauth} GET (handler::health::liveness),
        / "signup" => POST (handler::signup),
        / "me" {
            / => GET (handler::me),
            / "project" / "list" => GET (handler::me::project::list),
        },
        / "project" {
            / "get" => GET (handler::project::get),
            / "get-by-display-id" => GET (handler::project::get_by_display_id),
            / "check-display-id" => GET (handler::project::check_display_id),
            / "create" => POST (handler::project::create),
            / "update" => POST (handler::project::update),
            / "list" => GET (handler::project::list),
            / "export" => GET (handler::project::export),
            / "form" {
                / "get" => GET (handler::project::form::get),
                / "list" => GET (handler::project::form::list),
                / "answer" {
                    / => POST (handler::project::form::answer),
                    / "get" => GET (handler::project::form::answer::get),
                }
            }
        },
        / "form" {
            / "get" => GET (handler::form::get),
            / "list" => GET (handler::form::list),
            / "create" => POST (handler::form::create),
            / "answer" {
                / "list" => GET (handler::form::answer::list),
                / "export" => GET (handler::form::answer::export),
            }
        },
        / "form-answer" {
            / "get" => GET (handler::form_answer::get),
        },
        / "user" {
            / "get" => GET (handler::user::get),
            / "list" => GET (handler::user::list),
            / "export" => GET (handler::user::export),
            / "update" => POST (handler::user::update),
        }
    };

    routes
        .recover(handle_rejection)
        .with(warp::trace::request())
}
