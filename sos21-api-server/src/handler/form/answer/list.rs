use crate::app::Context;
use crate::handler::model::{form::FormId, form_answer::FormAnswer};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::list_form_answers;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub form_id: FormId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub answers: Vec<FormAnswer>,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    FormNotFound,
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::FormNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<list_form_answers::Error> for Error {
    fn from(err: list_form_answers::Error) -> Error {
        match err {
            list_form_answers::Error::FormNotFound => Error::FormNotFound,
            list_form_answers::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let answers = list_form_answers::run(&ctx, request.form_id.into_use_case()).await?;
    let answers = answers.into_iter().map(FormAnswer::from_use_case).collect();
    Ok(Response { answers })
}