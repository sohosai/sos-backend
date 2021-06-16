use crate::app::Context;
use crate::handler::model::{form::FormId, form_answer::FormAnswer, project::ProjectId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_project_form_answer;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub project_id: ProjectId,
    pub form_id: FormId,
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
    ProjectNotFound,
    FormNotFound,
    FormAnswerNotFound,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::ProjectNotFound => StatusCode::NOT_FOUND,
            Error::FormNotFound => StatusCode::NOT_FOUND,
            Error::FormAnswerNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<get_project_form_answer::Error> for Error {
    fn from(err: get_project_form_answer::Error) -> Error {
        match err {
            get_project_form_answer::Error::ProjectNotFound => Error::ProjectNotFound,
            get_project_form_answer::Error::FormNotFound => Error::FormNotFound,
            get_project_form_answer::Error::FormAnswerNotFound => Error::FormAnswerNotFound,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let answer = get_project_form_answer::run(
        &ctx,
        request.project_id.into_use_case(),
        request.form_id.into_use_case(),
    )
    .await?;
    let answer = FormAnswer::from_use_case(answer);
    Ok(Response { answer })
}
