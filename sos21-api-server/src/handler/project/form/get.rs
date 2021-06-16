use crate::app::Context;
use crate::handler::model::{
    form::{Form, FormId},
    project::ProjectId,
};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_project_form;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub project_id: ProjectId,
    pub form_id: FormId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
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
    FormNotFound,
    ProjectNotFound,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::FormNotFound => StatusCode::NOT_FOUND,
            Error::ProjectNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<get_project_form::Error> for Error {
    fn from(err: get_project_form::Error) -> Error {
        match err {
            get_project_form::Error::FormNotFound => Error::FormNotFound,
            get_project_form::Error::ProjectNotFound => Error::ProjectNotFound,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let form = get_project_form::run(
        &ctx,
        request.project_id.into_use_case(),
        request.form_id.into_use_case(),
    )
    .await?;
    let form = Form::from_use_case(form);
    Ok(Response { form })
}
