use crate::app::Context;
use crate::handler::model::user::{User, UserId, UserKanaName, UserName, UserRole};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain_context::Login;
use sos21_use_case::update_user;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub id: UserId,
    #[serde(default)]
    pub name: Option<UserName>,
    #[serde(default)]
    pub kana_name: Option<UserKanaName>,
    #[serde(default)]
    pub phone_number: Option<String>,
    #[serde(default)]
    pub affiliation: Option<String>,
    #[serde(default)]
    pub role: Option<UserRole>,
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
pub enum Error {
    NotFound,
    InsufficientPermissions,
    InvalidField(&'static str),
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
            Error::InvalidField(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<update_user::Error> for Error {
    fn from(err: update_user::Error) -> Error {
        match err {
            update_user::Error::NotFound => Error::NotFound,
            update_user::Error::InsufficientPermissions => Error::InsufficientPermissions,
            update_user::Error::InvalidUserName => Error::InvalidField("name"),
            update_user::Error::InvalidUserKanaName => Error::InvalidField("kana_name"),
            update_user::Error::InvalidPhoneNumber => Error::InvalidField("phone_number"),
            update_user::Error::InvalidUserAffiliation => Error::InvalidField("affiliation"),
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = update_user::Input {
        id: request.id.into_use_case(),
        name: request.name.map(UserName::into_use_case),
        kana_name: request.kana_name.map(UserKanaName::into_use_case),
        phone_number: request.phone_number,
        affiliation: request.affiliation,
        role: request.role.map(UserRole::into_use_case),
    };
    let user = update_user::run(&ctx, input).await?;
    let user = User::from_use_case(user);
    Ok(Response { user })
}
