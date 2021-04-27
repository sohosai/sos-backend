use crate::app::Context;
use crate::handler::model::pending_project::PendingProjectId;
use crate::handler::model::project::Project;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::accept_project_subowner;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub pending_project_id: PendingProjectId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub project: Project,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::CREATED
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    PendingProjectNotFound,
    TooManyProjects,
    NotAnsweredRegistrationForm,
    SameOwnerSubowner,
    AlreadyProjectOwner,
    AlreadyProjectSubowner,
    AlreadyPendingProjectOwner,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::PendingProjectNotFound => StatusCode::NOT_FOUND,
            Error::TooManyProjects => StatusCode::CONFLICT,
            Error::NotAnsweredRegistrationForm => StatusCode::CONFLICT,
            Error::SameOwnerSubowner => StatusCode::CONFLICT,
            Error::AlreadyProjectOwner => StatusCode::CONFLICT,
            Error::AlreadyProjectSubowner => StatusCode::CONFLICT,
            Error::AlreadyPendingProjectOwner => StatusCode::CONFLICT,
        }
    }
}

impl From<accept_project_subowner::Error> for Error {
    fn from(err: accept_project_subowner::Error) -> Error {
        match err {
            accept_project_subowner::Error::PendingProjectNotFound => Error::PendingProjectNotFound,
            accept_project_subowner::Error::TooManyProjects => Error::TooManyProjects,
            accept_project_subowner::Error::NotAnsweredRegistrationForm => {
                Error::NotAnsweredRegistrationForm
            }
            accept_project_subowner::Error::SameOwnerSubowner => Error::SameOwnerSubowner,
            accept_project_subowner::Error::AlreadyProjectOwner => Error::AlreadyProjectOwner,
            accept_project_subowner::Error::AlreadyProjectSubowner => Error::AlreadyProjectSubowner,
            accept_project_subowner::Error::AlreadyPendingProjectOwner => {
                Error::AlreadyPendingProjectOwner
            }
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let pending_project_id = request.pending_project_id.into_use_case();
    let project = accept_project_subowner::run(&ctx, pending_project_id).await?;
    let project = Project::from_use_case(project);
    Ok(Response { project })
}
