use crate::app::Context;
use crate::handler::model::user::{User, UserKanaName, UserName};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain_context::Authentication;
use sos21_use_case::signup;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub name: UserName,
    pub kana_name: UserKanaName,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub user: User,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::CREATED
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Error {
    InvalidField(&'static str),
    AlreadySignedUp,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidField(_) => StatusCode::BAD_REQUEST,
            Error::AlreadySignedUp => StatusCode::CONFLICT,
        }
    }
}

impl From<signup::Error> for Error {
    fn from(err: signup::Error) -> Error {
        match err {
            signup::Error::AlreadySignedUp => Error::AlreadySignedUp,
            signup::Error::InvalidUserName => Error::InvalidField("name"),
            signup::Error::InvalidUserKanaName => Error::InvalidField("kana_name"),
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(
    ctx: Authentication<Context>,
    request: Request,
) -> HandlerResult<Response, Error> {
    let input = signup::Input {
        name: request.name.into_use_case(),
        kana_name: request.kana_name.into_use_case(),
    };
    let user = signup::run(&ctx, input).await?;
    let user = User::from_use_case(user);
    Ok(Response { user })
}
