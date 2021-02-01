use crate::app::Context;
use crate::handler::model::project::Project;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_project_by_display_id;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub display_id: String,
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
    InvalidField { field: &'static str },
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::ProjectNotFound => StatusCode::NOT_FOUND,
            Error::InvalidField { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<get_project_by_display_id::Error> for Error {
    fn from(err: get_project_by_display_id::Error) -> Error {
        match err {
            get_project_by_display_id::Error::NotFound => Error::ProjectNotFound,
            get_project_by_display_id::Error::InvalidDisplayId => Error::InvalidField {
                field: "display_id",
            },
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let project = get_project_by_display_id::run(&ctx, request.display_id).await?;
    let project = Project::from_use_case(project);
    Ok(Response { project })
}
