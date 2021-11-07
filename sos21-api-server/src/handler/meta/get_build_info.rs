use crate::build_info;
use crate::handler::{HandlerError, HandlerResponse, HandlerResult};

use anyhow::anyhow;
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

fn git_info() -> Option<ResponseGit> {
    let commit = build_info::GIT_COMMIT_HASH?;
    let version = build_info::GIT_VERSION?;
    let branch = build_info::GIT_HEAD_REF?;
    Some(ResponseGit {
        commit,
        version,
        branch,
    })
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(_request: Request) -> HandlerResult<Response, Error> {
    let git = match git_info() {
        Some(git) => git,
        None => {
            return Err(HandlerError::ServiceUnavailable(anyhow!(
                "no git info available"
            )))
        }
    };

    let version = build_info::PKG_VERSION;
    let profile = build_info::PROFILE;
    let out = option_env!("out");

    Ok(Response {
        version,
        profile,
        out,
        git,
    })
}
