use crate::app::Context;
use crate::handler::model::project::{Project, ProjectAttribute, ProjectCategory, ProjectId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::update_project;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub id: ProjectId,
    #[serde(default)]
    pub display_id: Option<String>,
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
    pub category: Option<ProjectCategory>,
    #[serde(default)]
    pub attributes: Option<Vec<ProjectAttribute>>,
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
    InsufficientPermissions,
    UnavailableProjectDisplayId,
    DuplicatedProjectAttributes,
    InvalidField { field: &'static str },
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::ProjectNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
            Error::UnavailableProjectDisplayId => StatusCode::CONFLICT,
            Error::DuplicatedProjectAttributes => StatusCode::BAD_REQUEST,
            Error::InvalidField { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<update_project::Error> for Error {
    fn from(err: update_project::Error) -> Error {
        match err {
            update_project::Error::NotFound => Error::ProjectNotFound,
            update_project::Error::InsufficientPermissions => Error::InsufficientPermissions,
            update_project::Error::InvalidDisplayId => Error::InvalidField {
                field: "display_id",
            },
            update_project::Error::UnavailableDisplayId => Error::UnavailableProjectDisplayId,
            update_project::Error::InvalidName => Error::InvalidField { field: "name" },
            update_project::Error::InvalidKanaName => Error::InvalidField { field: "kana_name" },
            update_project::Error::InvalidGroupName => Error::InvalidField {
                field: "group_name",
            },
            update_project::Error::InvalidKanaGroupName => Error::InvalidField {
                field: "kana_group_name",
            },
            update_project::Error::InvalidDescription => Error::InvalidField {
                field: "description",
            },
            update_project::Error::DuplicatedAttributes => Error::DuplicatedProjectAttributes,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = update_project::Input {
        id: request.id.into_use_case(),
        display_id: request.display_id,
        name: request.name,
        kana_name: request.kana_name,
        group_name: request.group_name,
        kana_group_name: request.kana_group_name,
        description: request.description,
        category: request.category.map(ProjectCategory::into_use_case),
        attributes: request.attributes.map(|attributes| {
            attributes
                .into_iter()
                .map(ProjectAttribute::into_use_case)
                .collect()
        }),
    };
    let project = update_project::run(&ctx, input).await?;
    let project = Project::from_use_case(project);
    Ok(Response { project })
}
