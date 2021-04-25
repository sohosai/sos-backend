use crate::app::Context;
use crate::handler::model::{
    pending_project::PendingProjectId, registration_form::RegistrationForm,
};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::list_pending_project_registration_forms;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub pending_project_id: PendingProjectId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub registration_forms: Vec<RegistrationForm>,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    PendingProjectNotFound,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::PendingProjectNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<list_pending_project_registration_forms::Error> for Error {
    fn from(err: list_pending_project_registration_forms::Error) -> Error {
        match err {
            list_pending_project_registration_forms::Error::NotFound => {
                Error::PendingProjectNotFound
            }
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let registration_forms = list_pending_project_registration_forms::run(
        &ctx,
        request.pending_project_id.into_use_case(),
    )
    .await?;
    let registration_forms = registration_forms
        .into_iter()
        .map(RegistrationForm::from_use_case)
        .collect();
    Ok(Response { registration_forms })
}
