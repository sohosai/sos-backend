use crate::app::Context;
use crate::handler::model::{
    registration_form::RegistrationFormId, registration_form_answer::RegistrationFormAnswer,
};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::list_registration_form_answers;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub registration_form_id: RegistrationFormId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub answers: Vec<RegistrationFormAnswer>,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    RegistrationFormNotFound,
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::RegistrationFormNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<list_registration_form_answers::Error> for Error {
    fn from(err: list_registration_form_answers::Error) -> Error {
        match err {
            list_registration_form_answers::Error::RegistrationFormNotFound => {
                Error::RegistrationFormNotFound
            }
            list_registration_form_answers::Error::InsufficientPermissions => {
                Error::InsufficientPermissions
            }
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let registration_form_id = request.registration_form_id.into_use_case();
    let answers = list_registration_form_answers::run(&ctx, registration_form_id).await?;
    let answers = answers
        .into_iter()
        .map(RegistrationFormAnswer::from_use_case)
        .collect();
    Ok(Response { answers })
}
