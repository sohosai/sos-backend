use crate::app::Context;
use crate::handler::model::user::{User, UserId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_user;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub user_id: UserId,
}

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
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    UserNotFound,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::UserNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<get_user::Error> for Error {
    fn from(err: get_user::Error) -> Error {
        match err {
            get_user::Error::NotFound => Error::UserNotFound,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let user = get_user::run(&ctx, request.user_id.into_use_case()).await?;
    Ok(Response {
        user: User::from_use_case(user),
    })
}
