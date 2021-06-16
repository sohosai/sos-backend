use crate::app::Context;
use crate::handler::model::distributed_file::DistributedFile;
use crate::handler::model::project::ProjectId;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::list_distributed_files;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub distributed_files: Vec<DistributedFile>,
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

impl From<list_distributed_files::Error> for Error {
    fn from(err: list_distributed_files::Error) -> Error {
        match err {
            list_distributed_files::Error::ProjectNotFound => Error::ProjectNotFound,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let distributed_files =
        list_distributed_files::run(&ctx, request.project_id.into_use_case()).await?;
    let distributed_files = distributed_files
        .into_iter()
        .map(DistributedFile::from_use_case)
        .collect();
    Ok(Response { distributed_files })
}
