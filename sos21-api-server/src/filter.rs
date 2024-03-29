use std::convert::Infallible;

use crate::app::App;

use mime::Mime;
use warp::{
    http::{header, method::Method},
    Filter,
};

mod authentication;
mod error;

use authentication::authenticate;
pub use authentication::{AuthenticationInfo, KeyStore};
pub use error::model;
use error::{handle_cors_rejection, handle_rejection};

macro_rules! route {
    (@way GET) => { warp::get().and(warp::query()) };
    (@way POST) => { warp::post().and(warp::body::json()) };
    (@way POST_STREAM) => {
        warp::post()
            .and(warp::header::<Mime>("content-type"))
            .and(warp::body::stream())
    };
    (@path) => { warp::any() };
    (@path $name:literal) => { warp::path($name) };
    (@options $with_auth:ident, $with_app:ident, {noapp}) => { warp::any() };
    (@options $with_auth:ident, $with_app:ident, {noauth}) => { $with_app.clone() };
    (@options $with_auth:ident, $with_app:ident, {}) => { $with_app.clone().and($with_auth.clone()) };
    ($with_auth:ident, $with_app:ident, / $name:literal { $($inner:tt)+ }) => {
        warp::path($name)
            .and(routes!{ $with_auth, $with_app, $($inner)+ })
    };
    ($with_auth:ident, $with_app:ident,
       / $name_1:literal $(/ $name_n:literal)+ => {$($options:tt),*} $way:ident ($handler:path)
    ) => {
        warp::path($name_1)
            .and(route!{ $with_auth, $with_app, $(/ $name_n)+ => {$($options),*} $way ($handler) })
    };
    ($with_auth:ident, $with_app:ident,
       / $($name:literal)? => {$($options:tt),*} $way:ident ($handler:path)
    ) => {
        route!(@path $($name)?)
            .and(warp::path::end()
                .and(route!(@options $with_auth, $with_app, {$($options),*}))
                .and(route!(@way $way))
                .and_then($handler))
    };
}

macro_rules! routes {
    ($with_auth:ident, $with_app:ident,
       / $($name_1:literal)/ * $({ $($inner_1:tt)+ })? $(=> $({$($option_1:tt),*})? $way_1:ident ($handler_1:path) )?
         $(, / $($name_n:literal)/ * $({ $($inner_n:tt)+ })? $(=> $({$($option_n:tt),*})? $way_n:ident ($handler_n:path) )? )*
         $(,)?
    ) => {
        route!{ $with_auth, $with_app, / $($name_1)/ * $({ $($inner_1)+ })? $(=> {$($($option_1),*)?} $way_1 ($handler_1))? }
            $( .or(route!{
                    $with_auth, $with_app, / $($name_n)/ * $({ $($inner_n)+ })? $(=> {$($($option_n),*)?} $way_n ($handler_n))?
                })
                .boxed()  // workaround for seanmonstar/warp#811
            )*
    }
}

pub fn endpoints(
    app: App,
    key_store: KeyStore,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Infallible> + Clone {
    use crate::handler;

    let with_auth = authenticate(key_store, app.config().clone());
    let with_app = warp::any().map(move || app.clone());

    let routes = routes! { with_auth, with_app,
        / "meta" {
            / "get-build-info" => {noapp} GET (handler::meta::get_build_info),
            / "health" {
                / "check" => {noauth} GET (handler::meta::health::check),
                / "check-liveness" => {noapp} GET (handler::meta::health::check_liveness),
            },
        },
        / "signup" => POST (handler::signup),
        / "me" {
            / "get" => GET (handler::me::get),
            / "project" / "get" => GET (handler::me::project::get),
            / "pending-project" / "get" => GET (handler::me::pending_project::get),
            / "file" {
                / "list" => GET (handler::me::file::list),
                / "check-usage" => GET (handler::me::file::check_usage),
            },
            / "file-sharing" / "list" => GET (handler::me::file_sharing::list),
        },
        / "get-project-creation-availability" =>{noauth}  GET(handler::project_creation_availability::get),
        / "project" {
            / "prepare" => POST (handler::project::prepare),
            / "create" => POST (handler::project::create),
            / "get" => GET (handler::project::get),
            / "update" => POST (handler::project::update),
            / "update-any" => POST (handler::project::update_any),
            / "list" => GET (handler::project::list),
            / "export" => GET (handler::project::export),
            / "form" {
                / "get" => GET (handler::project::form::get),
                / "list" => GET (handler::project::form::list),
                / "answer" {
                    / => POST (handler::project::form::answer),
                    / "get" => GET (handler::project::form::answer::get),
                    / "update" => POST (handler::project::form::answer::update),
                    / "file-sharing" {
                        / "get-file" => GET (handler::project::form::answer::file_sharing::get_file),
                        / "get-file-info" => GET (handler::project::form::answer::file_sharing::get_file_info),
                    }
                }
            },
            / "registration-form" {
                / "get" => GET (handler::project::registration_form::get),
                / "list" => GET (handler::project::registration_form::list),
                / "answer" {
                    / "get" => GET (handler::project::registration_form::answer::get),
                    / "update" => POST (handler::project::registration_form::answer::update),
                    / "file-sharing" {
                        / "get-file" => GET (handler::project::registration_form::answer::file_sharing::get_file),
                        / "get-file-info" => GET (handler::project::registration_form::answer::file_sharing::get_file_info),
                    }
                }
            },
            / "file-sharing" {
                / "get-file" => GET (handler::project::file_sharing::get_file),
                / "get-file-info" => GET (handler::project::file_sharing::get_file_info),
            },
            / "file-distribution" {
                / "list" => GET (handler::project::file_distribution::list),
                / "get" => GET (handler::project::file_distribution::get),
            }
        },
        / "pending-project" {
            / "get" => GET (handler::pending_project::get),
            / "update" => POST (handler::pending_project::update),
            / "update-any" => POST (handler::pending_project::update_any),
            / "registration-form" {
                / "get" => GET (handler::pending_project::registration_form::get),
                / "list" => GET (handler::pending_project::registration_form::list),
                / "answer" {
                    / => POST (handler::pending_project::registration_form::answer),
                    / "update" => POST (handler::pending_project::registration_form::answer::update),
                    / "get" => GET (handler::pending_project::registration_form::answer::get),
                }
            }
        },
        / "form" {
            / "get" => GET (handler::form::get),
            / "list" => GET (handler::form::list),
            / "create" => POST (handler::form::create),
            / "update" => POST (handler::form::update),
            / "answer" {
                / "list" => GET (handler::form::answer::list),
                / "export" => GET (handler::form::answer::export),
            }
        },
        / "form-answer" {
            / "get" => GET (handler::form_answer::get),
            / "file-sharing" {
                / "get-file" => GET (handler::form_answer::file_sharing::get_file),
                / "get-file-info" => GET (handler::form_answer::file_sharing::get_file_info),
            }
        },
        / "user" {
            / "get" => GET (handler::user::get),
            / "list" => GET (handler::user::list),
            / "export" => GET (handler::user::export),
            / "update" => POST (handler::user::update),
        },
        / "file" {
            / "create" => POST_STREAM (handler::file::create),
            / "get" => GET (handler::file::get),
            / "get-info" => GET (handler::file::get_info),
            / "share" => POST (handler::file::share),
        },
        / "file-sharing" {
            / "get" => GET (handler::file_sharing::get),
            / "revoke" => POST (handler::file_sharing::revoke),
            / "get-file" => GET (handler::file_sharing::get_file),
            / "get-file-info" => GET (handler::file_sharing::get_file_info),
            / "get-public-file" => {noauth} GET (handler::file_sharing::get_public_file),
            / "get-public-file-info" => {noauth} GET (handler::file_sharing::get_public_file_info),
        },
        / "file-distribution" {
            / "create" => POST (handler::file_distribution::create),
            / "list" => GET (handler::file_distribution::list),
            / "get" => GET (handler::file_distribution::get),
        },
        / "registration-form" {
            / "get" => GET (handler::registration_form::get),
            / "list" => GET (handler::registration_form::list),
            / "create" => POST (handler::registration_form::create),
            / "answer" {
                / "list" => GET (handler::registration_form::answer::list),
                / "export" => GET (handler::registration_form::answer::export),
            }
        },
        / "registration-form-answer" {
            / "get" => GET (handler::registration_form_answer::get),
            / "file-sharing" {
                / "get-file" => GET (handler::registration_form_answer::file_sharing::get_file),
                / "get-file-info" => GET (handler::registration_form_answer::file_sharing::get_file_info),
            }
        },
        / "invite-user" => POST (handler::invite_user),
        / "user-invitation" {
            / "get" => GET (handler::user_invitation::get),
            / "list" => GET (handler::user_invitation::list),
            / "delete" => POST (handler::user_invitation::delete),
        },
        / "assign-user-role-to-email" => POST (handler::assign_user_role_to_email),
    };

    let cors = warp::cors()
        .allow_any_origin()
        .allow_method(Method::GET)
        .allow_method(Method::POST)
        .allow_header(header::AUTHORIZATION)
        .allow_header(header::CONTENT_TYPE)
        .expose_header(header::CONTENT_DISPOSITION)
        .max_age(std::time::Duration::from_secs(30 * 60));

    routes
        .recover(handle_rejection)
        .with(cors)
        .recover(handle_cors_rejection)
        .with(warp::trace::request())
}
