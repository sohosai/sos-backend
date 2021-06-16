use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub version: &'static str,
    pub profile: &'static str,
    pub out: Option<&'static str>,
    pub git: ResponseGit,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseGit {
    pub commit: &'static str,
    pub version: &'static str,
    pub branch: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(_request: Request) -> HandlerResult<Response, Error> {
    // See https://docs.rs/vergen/ and sos21-api-server/build.rs

    let git = {
        let commit = env!("VERGEN_GIT_SHA");
        let version = env!("VERGEN_GIT_SEMVER");
        let branch = env!("VERGEN_GIT_BRANCH");
        ResponseGit {
            commit,
            version,
            branch,
        }
    };

    let version = env!("VERGEN_BUILD_SEMVER");
    let profile = env!("VERGEN_CARGO_PROFILE");
    let out = option_env!("out");

    Ok(Response {
        version,
        profile,
        out,
        git,
    })
}
