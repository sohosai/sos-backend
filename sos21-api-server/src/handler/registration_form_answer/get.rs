use crate::app::Context;
use crate::handler::model::registration_form_answer::{
    RegistrationFormAnswer, RegistrationFormAnswerId,
};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_registration_form_answer;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub answer_id: RegistrationFormAnswerId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub answer: RegistrationFormAnswer,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    RegistrationFormAnswerNotFound,
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::RegistrationFormAnswerNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<get_registration_form_answer::Error> for Error {
    fn from(err: get_registration_form_answer::Error) -> Error {
        match err {
            get_registration_form_answer::Error::NotFound => Error::RegistrationFormAnswerNotFound,
            get_registration_form_answer::Error::InsufficientPermissions => {
                Error::InsufficientPermissions
            }
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let answer = get_registration_form_answer::run(&ctx, request.answer_id.into_use_case()).await?;
    let answer = RegistrationFormAnswer::from_use_case(answer);
    Ok(Response { answer })
}
