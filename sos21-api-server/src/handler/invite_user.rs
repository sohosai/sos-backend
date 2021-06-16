use crate::app::Context;
use crate::handler::model::user_invitation::{UserInvitation, UserInvitationRole};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::invite_user;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub email: String,
    pub role: UserInvitationRole,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub invitation: UserInvitation,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::CREATED
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    InvalidEmailAddress,
    NotUniversityEmailAddress,
    AlreadyInvitedEmailAddress,
    AlreadySignedUpEmailAddress,
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidEmailAddress => StatusCode::BAD_REQUEST,
            Error::NotUniversityEmailAddress => StatusCode::BAD_REQUEST,
            Error::AlreadyInvitedEmailAddress => StatusCode::CONFLICT,
            Error::AlreadySignedUpEmailAddress => StatusCode::CONFLICT,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<invite_user::Error> for Error {
    fn from(err: invite_user::Error) -> Error {
        match err {
            invite_user::Error::InvalidEmailAddress => Error::InvalidEmailAddress,
            invite_user::Error::NotUniversityEmailAddress => Error::NotUniversityEmailAddress,
            invite_user::Error::AlreadyInvitedEmailAddress => Error::AlreadyInvitedEmailAddress,
            invite_user::Error::AlreadySignedUpEmailAddress => Error::AlreadySignedUpEmailAddress,
            invite_user::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = invite_user::Input {
        email: request.email,
        role: request.role.into_use_case(),
    };
    let invitation = invite_user::run(&ctx, input).await?;
    let invitation = UserInvitation::from_use_case(invitation);
    Ok(Response { invitation })
}
