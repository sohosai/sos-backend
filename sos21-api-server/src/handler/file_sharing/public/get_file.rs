use crate::app::Context;
use crate::handler::model::file_sharing::FileSharingId;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_use_case::get_publicly_shared_file_object;
use warp::{
    http::{self, header, StatusCode},
    hyper::Body,
    reply,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub sharing_id: FileSharingId,
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

impl From<get_publicly_shared_file_object::Error> for Error {
    fn from(err: get_publicly_shared_file_object::Error) -> Error {
        match err {
            get_publicly_shared_file_object::Error::NotFound => Error::FileSharingNotFound,
            get_publicly_shared_file_object::Error::InvalidSharing => Error::InvalidFileSharing,
        }
    }
}

#[apply_macro::apply(raw_response_handler)]
pub async fn handler(ctx: Context, request: Request) -> HandlerResult<impl warp::Reply, Error> {
    let file_object =
        get_publicly_shared_file_object::run(&ctx, request.sharing_id.into_use_case()).await?;

    let reply = http::Response::new(Body::wrap_stream(file_object.object_data));
    let reply = reply::with_header(
        reply,
        header::CONTENT_TYPE,
        file_object.file.type_.to_string(),
    );
    let reply = reply::with_header(
        reply,
        header::CONTENT_LENGTH,
        file_object.file.size.to_string(),
    );
    let reply = reply::with_status(reply, StatusCode::OK);
    Ok(reply)
}
