use crate::app::Context;
use crate::handler::model::file::{File, FileId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_file;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub file_id: FileId,
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
    FileNotFound,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::FileNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<get_file::Error> for Error {
    fn from(err: get_file::Error) -> Error {
        match err {
            get_file::Error::NotFound => Error::FileNotFound,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let file = get_file::run(&ctx, request.file_id.into_use_case()).await?;
    let file = File::from_use_case(file);
    Ok(Response { file })
}
