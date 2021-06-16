use crate::app::Context;
use crate::handler::model::user::User;
use crate::handler::model::user_invitation::{UserInvitation, UserInvitationRole};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::assign_user_role_to_email;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub email: String,
    pub role: UserInvitationRole,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    Invitation(UserInvitation),
    User(User),
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    InvalidEmailAddress,
    NotUniversityEmailAddress,
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidEmailAddress => StatusCode::BAD_REQUEST,
            Error::NotUniversityEmailAddress => StatusCode::BAD_REQUEST,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<assign_user_role_to_email::Error> for Error {
    fn from(err: assign_user_role_to_email::Error) -> Error {
        match err {
            assign_user_role_to_email::Error::InvalidEmailAddress => Error::InvalidEmailAddress,
            assign_user_role_to_email::Error::NotUniversityEmailAddress => {
                Error::NotUniversityEmailAddress
            }
            assign_user_role_to_email::Error::InsufficientPermissions => {
                Error::InsufficientPermissions
            }
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = assign_user_role_to_email::Input {
        email: request.email,
        role: request.role.into_use_case(),
    };
    let output = assign_user_role_to_email::run(&ctx, input).await?;
    let response = match output {
        assign_user_role_to_email::Output::Invitation(invitation) => {
            Response::Invitation(UserInvitation::from_use_case(invitation))
        }
        assign_user_role_to_email::Output::User(user) => Response::User(User::from_use_case(user)),
    };
    Ok(response)
}
