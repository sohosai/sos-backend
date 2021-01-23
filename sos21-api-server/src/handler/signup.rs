use crate::app::{App, Authentication};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_model::user::{User, UserName};
use sos21_use_case as use_case;
use sos21_use_case::signup;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub name: UserName,
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
    AlreadySignedUp,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::AlreadySignedUp => StatusCode::CONFLICT,
        }
    }
}

impl From<signup::Error> for Error {
    fn from(err: signup::Error) -> Error {
        match err {
            signup::Error::AlreadySignedUp => Error::AlreadySignedUp,
        }
    }
}

pub async fn handler(app: Authentication<App>, request: Request) -> HandlerResult<Response, Error> {
    let user = use_case::signup::run(app, request.name).await?;
    Ok(Response { user })
}
