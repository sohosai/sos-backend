use crate::app::Context;
use crate::handler::model::file_sharing::{FileSharing, FileSharingId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_file_sharing;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub sharing_id: FileSharingId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub sharing: FileSharing,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    FileSharingNotFound,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::FileSharingNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<get_file_sharing::Error> for Error {
    fn from(err: get_file_sharing::Error) -> Error {
        match err {
            get_file_sharing::Error::NotFound => Error::FileSharingNotFound,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let sharing = get_file_sharing::run(&ctx, request.sharing_id.into_use_case()).await?;
    let sharing = FileSharing::from_use_case(sharing);
    Ok(Response { sharing })
}
