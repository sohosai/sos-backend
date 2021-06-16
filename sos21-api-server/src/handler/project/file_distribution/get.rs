use crate::app::Context;
use crate::handler::model::distributed_file::DistributedFile;
use crate::handler::model::file_distribution::FileDistributionId;
use crate::handler::model::project::ProjectId;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_distributed_file;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub project_id: ProjectId,
    pub distribution_id: FileDistributionId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub distributed_file: DistributedFile,
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
    FileDistributionNotFound,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::ProjectNotFound => StatusCode::NOT_FOUND,
            Error::FileDistributionNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<get_distributed_file::Error> for Error {
    fn from(err: get_distributed_file::Error) -> Error {
        match err {
            get_distributed_file::Error::ProjectNotFound => Error::ProjectNotFound,
            get_distributed_file::Error::FileDistributionNotFound => {
                Error::FileDistributionNotFound
            }
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = get_distributed_file::Input {
        project_id: request.project_id.into_use_case(),
        distribution_id: request.distribution_id.into_use_case(),
    };
    let distributed_file = get_distributed_file::run(&ctx, input).await?;
    let distributed_file = DistributedFile::from_use_case(distributed_file);
    Ok(Response { distributed_file })
}
