use std::convert::Infallible;

use crate::app::Context;
use crate::handler::model::user::User;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::Serialize;
use sos21_domain_context::Login;
use sos21_use_case::get_login_user;
use warp::http::StatusCode;

pub mod project;

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub user: User,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Error {}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl From<Infallible> for Error {
    fn from(x: Infallible) -> Error {
        match x {}
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>) -> HandlerResult<Response, Error> {
    let user = get_login_user::run(&ctx).await?;
    let user = User::from_use_case(user);
    Ok(Response { user })
}
