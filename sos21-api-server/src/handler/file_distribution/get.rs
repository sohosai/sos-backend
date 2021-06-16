use crate::app::Context;
use crate::handler::model::file_distribution::{FileDistribution, FileDistributionId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_file_distribution;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub distribution_id: FileDistributionId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub distribution: FileDistribution,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    FileDistributionNotFound,
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::FileDistributionNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<get_file_distribution::Error> for Error {
    fn from(err: get_file_distribution::Error) -> Error {
        match err {
            get_file_distribution::Error::NotFound => Error::FileDistributionNotFound,
            get_file_distribution::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let distribution_id = request.distribution_id.into_use_case();
    let distribution = get_file_distribution::run(&ctx, distribution_id).await?;
    let distribution = FileDistribution::from_use_case(distribution);
    Ok(Response { distribution })
}
