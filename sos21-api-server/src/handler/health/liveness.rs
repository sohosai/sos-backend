use crate::handler::{HandlerResponse, HandlerResult};

use serde::Serialize;
use warp::http::StatusCode;

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum Error {}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

pub async fn handler() -> HandlerResult<Response, Error> {
    Ok(Response { ok: true })
}
