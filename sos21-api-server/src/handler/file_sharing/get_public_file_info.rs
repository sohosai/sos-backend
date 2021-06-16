use crate::app::Context;
use crate::handler::model::file::File;
use crate::handler::model::file_sharing::FileSharingId;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_use_case::get_publicly_shared_file;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub sharing_id: FileSharingId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub file: File,
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
    InvalidFileSharing,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::FileSharingNotFound => StatusCode::NOT_FOUND,
            Error::InvalidFileSharing => StatusCode::FORBIDDEN,
        }
    }
}

impl From<get_publicly_shared_file::Error> for Error {
    fn from(err: get_publicly_shared_file::Error) -> Error {
        match err {
            get_publicly_shared_file::Error::NotFound => Error::FileSharingNotFound,
            get_publicly_shared_file::Error::InvalidSharing => Error::InvalidFileSharing,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Context, request: Request) -> HandlerResult<Response, Error> {
    let file = get_publicly_shared_file::run(&ctx, request.sharing_id.into_use_case()).await?;
    let file = File::from_use_case(file);
    Ok(Response { file })
}
