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
    pub registration_forms: Vec<ResponseRegistrationForm>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseRegistrationForm {
    pub has_answer: bool,
    #[serde(flatten)]
    pub registration_form: RegistrationForm,
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

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let registration_forms = list_pending_project_registration_forms::run(
        &ctx,
        request.pending_project_id.into_use_case(),
    )
    .await?;
    let registration_forms = registration_forms
        .into_iter()
        .map(|data| ResponseRegistrationForm {
            has_answer: data.has_answer,
            registration_form: RegistrationForm::from_use_case(data.registration_form),
        })
        .collect();
    Ok(Response { registration_forms })
}
