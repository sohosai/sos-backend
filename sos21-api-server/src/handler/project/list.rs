use crate::app::Context;
use crate::handler::model::project::Project;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::list_all_projects;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub projects: Vec<Project>,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<list_all_projects::Error> for Error {
    fn from(err: list_all_projects::Error) -> Error {
        match err {
            list_all_projects::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, _request: Request) -> HandlerResult<Response, Error> {
    let projects = list_all_projects::run(&ctx).await?;
    let projects = projects.into_iter().map(Project::from_use_case).collect();
    Ok(Response { projects })
}
