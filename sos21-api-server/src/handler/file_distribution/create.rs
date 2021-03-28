use crate::app::Context;
use crate::handler::model::file::FileId;
use crate::handler::model::file_distribution::FileDistribution;
use crate::handler::model::file_sharing::FileSharingId;
use crate::handler::model::project::ProjectId;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::distribute_files;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub name: String,
    pub description: String,
    pub files: Vec<RequestFileMapping>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestFileMapping {
    pub project_id: ProjectId,
    #[serde(flatten)]
    pub file: RequestFile,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestFile {
    FileId(FileId),
    SharingId(FileSharingId),
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub distribution: FileDistribution,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::CREATED
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    InvalidField { field: &'static str },
    ProjectNotFound,
    FileNotFound,
    FileSharingNotFound,
    InsufficientPermissions,
    NonSharableFile,
    OutOfScopeFileSharing,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidField { .. } => StatusCode::BAD_REQUEST,
            Error::ProjectNotFound => StatusCode::NOT_FOUND,
            Error::FileNotFound => StatusCode::NOT_FOUND,
            Error::FileSharingNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
            Error::NonSharableFile => StatusCode::FORBIDDEN,
            Error::OutOfScopeFileSharing => StatusCode::FORBIDDEN,
        }
    }
}

impl From<distribute_files::Error> for Error {
    fn from(err: distribute_files::Error) -> Error {
        match err {
            distribute_files::Error::InvalidName => Error::InvalidField { field: "name" },
            distribute_files::Error::InvalidDescription => Error::InvalidField {
                field: "description",
            },
            distribute_files::Error::NoFiles => Error::InvalidField { field: "files" },
            distribute_files::Error::TooManyFiles => Error::InvalidField { field: "files" },
            distribute_files::Error::DuplicatedProjectId(_) => {
                Error::InvalidField { field: "files" }
            }
            distribute_files::Error::ProjectNotFound => Error::ProjectNotFound,
            distribute_files::Error::FileNotFound => Error::FileNotFound,
            distribute_files::Error::FileSharingNotFound => Error::FileSharingNotFound,
            distribute_files::Error::InsufficientPermissions => Error::InsufficientPermissions,
            distribute_files::Error::NonSharableFile => Error::NonSharableFile,
            distribute_files::Error::OutOfScopeFileSharing => Error::OutOfScopeFileSharing,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = distribute_files::Input {
        name: request.name,
        description: request.description,
        files: request
            .files
            .into_iter()
            .map(to_input_file_mapping)
            .collect(),
    };
    let distribution = distribute_files::run(&ctx, input).await?;
    let distribution = FileDistribution::from_use_case(distribution);
    Ok(Response { distribution })
}

fn to_input_file_mapping(mapping: RequestFileMapping) -> distribute_files::InputFileMapping {
    distribute_files::InputFileMapping {
        project_id: mapping.project_id.into_use_case(),
        file: to_input_file(mapping.file),
    }
}

fn to_input_file(file: RequestFile) -> distribute_files::InputFile {
    match file {
        RequestFile::FileId(file_id) => distribute_files::InputFile::File(file_id.into_use_case()),
        RequestFile::SharingId(sharing_id) => {
            distribute_files::InputFile::Sharing(sharing_id.into_use_case())
        }
    }
}
