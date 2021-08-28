use crate::app::Context;
use crate::handler::model::pending_project::{PendingProject, PendingProjectId};
use crate::handler::model::project::ProjectAttribute;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::update_pending_project;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub id: PendingProjectId,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub kana_name: Option<String>,
    #[serde(default)]
    pub group_name: Option<String>,
    #[serde(default)]
    pub kana_group_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: Option<Vec<ProjectAttribute>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub pending_project: PendingProject,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    PendingProjectNotFound,
    InsufficientPermissions,
    DuplicatedProjectAttributes,
    OutOfProjectCreationPeriod,
    InvalidField { field: &'static str },
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::PendingProjectNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
            Error::DuplicatedProjectAttributes => StatusCode::BAD_REQUEST,
            Error::OutOfProjectCreationPeriod => StatusCode::CONFLICT,
            Error::InvalidField { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<update_pending_project::Error> for Error {
    fn from(err: update_pending_project::Error) -> Error {
        match err {
            update_pending_project::Error::NotFound => Error::PendingProjectNotFound,
            update_pending_project::Error::InsufficientPermissions => {
                Error::InsufficientPermissions
            }
            update_pending_project::Error::OutOfCreationPeriod => Error::OutOfProjectCreationPeriod,
            update_pending_project::Error::InvalidName => Error::InvalidField { field: "name" },
            update_pending_project::Error::InvalidKanaName => {
                Error::InvalidField { field: "kana_name" }
            }
            update_pending_project::Error::InvalidGroupName => Error::InvalidField {
                field: "group_name",
            },
            update_pending_project::Error::InvalidKanaGroupName => Error::InvalidField {
                field: "kana_group_name",
            },
            update_pending_project::Error::InvalidDescription => Error::InvalidField {
                field: "description",
            },
            update_pending_project::Error::DuplicatedAttributes => {
                Error::DuplicatedProjectAttributes
            }
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = update_pending_project::Input {
        id: request.id.into_use_case(),
        name: request.name,
        kana_name: request.kana_name,
        group_name: request.group_name,
        kana_group_name: request.kana_group_name,
        description: request.description,
        attributes: request.attributes.map(|attributes| {
            attributes
                .into_iter()
                .map(ProjectAttribute::into_use_case)
                .collect()
        }),
    };
    let pending_project = update_pending_project::run(&ctx, input).await?;
    let pending_project = PendingProject::from_use_case(pending_project);
    Ok(Response { pending_project })
}
