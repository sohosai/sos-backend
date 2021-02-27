use crate::app::Context;
use crate::handler::model::form_answer::{FormAnswer, FormAnswerId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_form_answer;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub answer_id: FormAnswerId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub answer: FormAnswer,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    FormAnswerNotFound,
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::FormAnswerNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<get_form_answer::Error> for Error {
    fn from(err: get_form_answer::Error) -> Error {
        match err {
            get_form_answer::Error::NotFound => Error::FormAnswerNotFound,
            get_form_answer::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let answer = get_form_answer::run(&ctx, request.answer_id.into_use_case()).await?;
    let answer = FormAnswer::from_use_case(answer);
    Ok(Response { answer })
}
