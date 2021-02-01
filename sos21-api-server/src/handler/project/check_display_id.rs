use std::convert::Infallible;

use crate::app::Context;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::check_project_display_id_status;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub display_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "status")]
pub enum Response {
    Invalid { reason: ResponseInvalidReason },
    Unavailable,
    Available,
}

impl Response {
    fn from_use_case(status: check_project_display_id_status::DisplayIdStatus) -> Self {
        use check_project_display_id_status::DisplayIdStatus;
        match status {
            DisplayIdStatus::Invalid { reason } => Response::Invalid {
                reason: ResponseInvalidReason::from_use_case(reason),
            },
            DisplayIdStatus::Unavailable => Response::Unavailable,
            DisplayIdStatus::Available => Response::Available,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResponseInvalidReason {
    TooLong,
    TooShort,
    ContainsDisallowedCharacter,
    StartsWithUnderscore,
}

impl ResponseInvalidReason {
    fn from_use_case(reason: check_project_display_id_status::DisplayIdInvalidReason) -> Self {
        use check_project_display_id_status::DisplayIdInvalidReason;
        match reason {
            DisplayIdInvalidReason::TooLong => ResponseInvalidReason::TooLong,
            DisplayIdInvalidReason::TooShort => ResponseInvalidReason::TooShort,
            DisplayIdInvalidReason::ContainsDisallowedCharacter => {
                ResponseInvalidReason::ContainsDisallowedCharacter
            }
            DisplayIdInvalidReason::StartsWithUnderscore => {
                ResponseInvalidReason::StartsWithUnderscore
            }
        }
    }
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl From<Infallible> for Error {
    fn from(x: Infallible) -> Error {
        match x {}
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let status = check_project_display_id_status::run(&ctx, request.display_id).await?;
    Ok(Response::from_use_case(status))
}
