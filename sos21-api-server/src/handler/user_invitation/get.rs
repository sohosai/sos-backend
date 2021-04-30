use crate::app::Context;
use crate::handler::model::user_invitation::{UserInvitation, UserInvitationId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_user_invitation;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub invitation_id: UserInvitationId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub invitation: UserInvitation,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    UserInvitationNotFound,
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::UserInvitationNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<get_user_invitation::Error> for Error {
    fn from(err: get_user_invitation::Error) -> Error {
        match err {
            get_user_invitation::Error::NotFound => Error::UserInvitationNotFound,
            get_user_invitation::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let invitation = get_user_invitation::run(&ctx, request.invitation_id.into_use_case()).await?;
    let invitation = UserInvitation::from_use_case(invitation);
    Ok(Response { invitation })
}
