use crate::app::Context;
use crate::handler::model::project::{Project, ProjectId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::{get_project, get_project_by_code};
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    #[serde(flatten)]
    pub specifier: RequestSpecifier,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestSpecifier {
    ProjectId(ProjectId),
    ProjectCode(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub project: Project,
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
    InvalidProjectCode,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::ProjectNotFound => StatusCode::NOT_FOUND,
            Error::InvalidProjectCode => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<get_project::Error> for Error {
    fn from(err: get_project::Error) -> Error {
        match err {
            get_project::Error::NotFound => Error::ProjectNotFound,
        }
    }
}

impl From<get_project_by_code::Error> for Error {
    fn from(err: get_project_by_code::Error) -> Error {
        match err {
            get_project_by_code::Error::NotFound => Error::ProjectNotFound,
            get_project_by_code::Error::InvalidCode => Error::InvalidProjectCode,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let project = match request.specifier {
        RequestSpecifier::ProjectId(id) => get_project::run(&ctx, id.into_use_case()).await?,
        RequestSpecifier::ProjectCode(code) => get_project_by_code::run(&ctx, code).await?,
    };
    let project = Project::from_use_case(project);
    Ok(Response { project })
}
