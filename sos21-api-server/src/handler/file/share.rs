use crate::app::Context;
use crate::handler::model::date_time::DateTime;
use crate::handler::model::file::FileId;
use crate::handler::model::file_sharing::FileSharing;
use crate::handler::model::project_query::ProjectQuery;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::share_file;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub file_id: FileId,
    pub expires_at: Option<DateTime>,
    pub scope: RequestFileSharingScope,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RequestFileSharingScope {
    ProjectQuery { query: ProjectQuery },
    Committee,
    CommitteeOperator,
    Public,
}

impl RequestFileSharingScope {
    fn into_use_case(self) -> share_file::InputFileSharingScope {
        match self {
            RequestFileSharingScope::ProjectQuery { query } => {
                share_file::InputFileSharingScope::ProjectQuery(query.into_use_case())
            }
            RequestFileSharingScope::Committee => share_file::InputFileSharingScope::Committee,
            RequestFileSharingScope::CommitteeOperator => {
                share_file::InputFileSharingScope::CommitteeOperator
            }
            RequestFileSharingScope::Public => share_file::InputFileSharingScope::Public,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub sharing: FileSharing,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::CREATED
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    InvalidProjectQuery,
    InsufficientPermissions,
    FileNotFound,
    NonSharableFile,
    InvalidFileExpirationDate,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidProjectQuery => StatusCode::BAD_REQUEST,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
            Error::FileNotFound => StatusCode::NOT_FOUND,
            Error::NonSharableFile => StatusCode::FORBIDDEN,
            Error::InvalidFileExpirationDate => StatusCode::CONFLICT,
        }
    }
}

impl From<share_file::Error> for Error {
    fn from(err: share_file::Error) -> Error {
        match err {
            share_file::Error::InvalidQuery(_) => Error::InvalidProjectQuery,
            share_file::Error::InsufficientPermissions => Error::InsufficientPermissions,
            share_file::Error::FileNotFound => Error::FileNotFound,
            share_file::Error::NonSharableFile => Error::NonSharableFile,
            share_file::Error::InvalidExpirationDate => Error::InvalidFileExpirationDate,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = share_file::Input {
        file_id: request.file_id.into_use_case(),
        expires_at: request
            .expires_at
            .map(|expires_at| expires_at.into_use_case()),
        scope: request.scope.into_use_case(),
    };
    let sharing = share_file::run(&ctx, input).await?;
    let sharing = FileSharing::from_use_case(sharing);
    Ok(Response { sharing })
}
