use crate::app::Context;
use crate::handler::model::pending_project::PendingProject;
use crate::handler::model::project::{ProjectAttribute, ProjectCategory};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::prepare_project;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: Vec<ProjectAttribute>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub pending_project: PendingProject,
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
    DuplicatedProjectAttributes,
    AlreadyProjectOwner,
    AlreadyProjectSubowner,
    AlreadyPendingProjectOwner,
    OutOfProjectCreationPeriod,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidField { .. } => StatusCode::BAD_REQUEST,
            Error::DuplicatedProjectAttributes => StatusCode::BAD_REQUEST,
            Error::AlreadyProjectOwner => StatusCode::CONFLICT,
            Error::AlreadyProjectSubowner => StatusCode::CONFLICT,
            Error::AlreadyPendingProjectOwner => StatusCode::CONFLICT,
            Error::OutOfProjectCreationPeriod => StatusCode::CONFLICT,
        }
    }
}

impl From<prepare_project::Error> for Error {
    fn from(err: prepare_project::Error) -> Error {
        match err {
            prepare_project::Error::InvalidName => Error::InvalidField { field: "name" },
            prepare_project::Error::InvalidKanaName => Error::InvalidField { field: "kana_name" },
            prepare_project::Error::InvalidGroupName => Error::InvalidField {
                field: "group_name",
            },
            prepare_project::Error::InvalidKanaGroupName => Error::InvalidField {
                field: "kana_group_name",
            },
            prepare_project::Error::InvalidDescription => Error::InvalidField {
                field: "description",
            },
            prepare_project::Error::DuplicatedAttributes => Error::DuplicatedProjectAttributes,
            prepare_project::Error::AlreadyProjectOwner => Error::AlreadyProjectOwner,
            prepare_project::Error::AlreadyProjectSubowner => Error::AlreadyProjectSubowner,
            prepare_project::Error::AlreadyPendingProjectOwner => Error::AlreadyPendingProjectOwner,
            prepare_project::Error::OutOfCreationPeriod => Error::OutOfProjectCreationPeriod,
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = prepare_project::Input {
        name: request.name,
        kana_name: request.kana_name,
        group_name: request.group_name,
        kana_group_name: request.kana_group_name,
        description: request.description,
        category: request.category.into_use_case(),
        attributes: request
            .attributes
            .into_iter()
            .map(ProjectAttribute::into_use_case)
            .collect(),
    };
    let pending_project = prepare_project::run(&ctx, input).await?;
    let pending_project = PendingProject::from_use_case(pending_project);
    Ok(Response { pending_project })
}
