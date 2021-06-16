use crate::app::Context;
use crate::handler::model::{form::Form, project::ProjectId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::list_project_forms;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub forms: Vec<ResponseForm>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseForm {
    pub has_answer: bool,
    #[serde(flatten)]
    pub form: Form,
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
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::ProjectNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<list_project_forms::Error> for Error {
    fn from(err: list_project_forms::Error) -> Error {
        match err {
            list_project_forms::Error::NotFound => Error::ProjectNotFound,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let forms = list_project_forms::run(&ctx, request.project_id.into_use_case()).await?;
    let forms = forms
        .into_iter()
        .map(|data| ResponseForm {
            has_answer: data.has_answer,
            form: Form::from_use_case(data.form),
        })
        .collect();
    Ok(Response { forms })
}
