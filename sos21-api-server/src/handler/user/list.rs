use crate::app::App;
use crate::handler::model::user::User;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain_context::Login;
use sos21_use_case::list_users;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub users: Vec<User>,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Error {
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<list_users::Error> for Error {
    fn from(err: list_users::Error) -> Error {
        match err {
            list_users::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

pub async fn handler(app: Login<App>, _request: Request) -> HandlerResult<Response, Error> {
    let users = list_users::run(&app).await?;
    let users = users.into_iter().map(User::from_use_case).collect();
    Ok(Response { users })
}
