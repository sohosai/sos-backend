use crate::app::{App, Authentication};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::Serialize;
use sos21_model::user::User;
use sos21_use_case as use_case;
use sos21_use_case::get_login_user;
use warp::http::StatusCode;

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
pub enum Error {
    NotSignedUp,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::NotSignedUp => StatusCode::FORBIDDEN,
        }
    }
}

impl From<get_login_user::Error> for Error {
    fn from(err: get_login_user::Error) -> Error {
        match err {
            get_login_user::Error::NotSignedUp => Error::NotSignedUp,
        }
    }
}

pub async fn handler(app: Authentication<App>) -> HandlerResult<Response, Error> {
    let user = use_case::get_login_user::run(app).await?;
    Ok(Response { user })
}
