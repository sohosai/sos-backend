use crate::app::Context;
use crate::handler::model::{
    project::ProjectId, registration_form::RegistrationFormId,
    registration_form_answer::RegistrationFormAnswer,
};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_project_registration_form_answer;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub project_id: ProjectId,
    pub registration_form_id: RegistrationFormId,
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
    ProjectNotFound,
    RegistrationFormNotFound,
    RegistrationFormAnswerNotFound,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::ProjectNotFound => StatusCode::NOT_FOUND,
            Error::RegistrationFormNotFound => StatusCode::NOT_FOUND,
            Error::RegistrationFormAnswerNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<get_project_registration_form_answer::Error> for Error {
    fn from(err: get_project_registration_form_answer::Error) -> Error {
        match err {
            get_project_registration_form_answer::Error::ProjectNotFound => Error::ProjectNotFound,
            get_project_registration_form_answer::Error::RegistrationFormNotFound => {
                Error::RegistrationFormNotFound
            }
            get_project_registration_form_answer::Error::RegistrationFormAnswerNotFound => {
                Error::RegistrationFormAnswerNotFound
            }
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let answer = get_project_registration_form_answer::run(
        &ctx,
        request.project_id.into_use_case(),
        request.registration_form_id.into_use_case(),
    )
    .await?;
    let answer = RegistrationFormAnswer::from_use_case(answer);
    Ok(Response { answer })
}
