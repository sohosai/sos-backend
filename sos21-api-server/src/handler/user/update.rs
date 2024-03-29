use crate::app::Context;
use crate::handler::model::user::{User, UserCategory, UserId, UserKanaName, UserName, UserRole};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::update_any_user;
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
    pub role: Option<UserRole>,
    #[serde(default)]
    pub category: Option<UserCategory>,
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
    InsufficientPermissions,
    InvalidField { field: &'static str },
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::UserNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
            Error::InvalidField { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<update_any_user::Error> for Error {
    fn from(err: update_any_user::Error) -> Error {
        match err {
            update_any_user::Error::NotFound => Error::UserNotFound,
            update_any_user::Error::InsufficientPermissions => Error::InsufficientPermissions,
            update_any_user::Error::InvalidName => Error::InvalidField { field: "name" },
            update_any_user::Error::InvalidKanaName => Error::InvalidField { field: "kana_name" },
            update_any_user::Error::InvalidPhoneNumber => Error::InvalidField {
                field: "phone_number",
            },
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = update_any_user::Input {
        id: request.id.into_use_case(),
        name: request.name.map(UserName::into_use_case),
        kana_name: request.kana_name.map(UserKanaName::into_use_case),
        phone_number: request.phone_number,
        role: request.role.map(UserRole::into_use_case),
        category: request.category.map(UserCategory::into_use_case),
    };
    let user = update_any_user::run(&ctx, input).await?;
    let user = User::from_use_case(user);
    Ok(Response { user })
}
