use crate::app::Context;
use crate::handler::model::pending_project::{PendingProject, PendingProjectId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_pending_project;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub pending_project_id: PendingProjectId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub pending_project: PendingProject,
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

impl From<get_pending_project::Error> for Error {
    fn from(err: get_pending_project::Error) -> Error {
        match err {
            get_pending_project::Error::NotFound => Error::ProjectNotFound,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let pending_project_id = request.pending_project_id.into_use_case();
    let pending_project = get_pending_project::run(&ctx, pending_project_id).await?;
    let pending_project = PendingProject::from_use_case(pending_project);
    Ok(Response { pending_project })
}
