use crate::app::App;
use crate::handler::model::project::{Project, ProjectAttribute, ProjectCategory, ProjectId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain_context::Login;
use sos21_use_case::update_project;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub id: ProjectId,
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
pub enum Error {
    NotFound,
    InsufficientPermissions,
    InvalidField(&'static str),
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
            Error::InvalidField(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<update_project::Error> for Error {
    fn from(err: update_project::Error) -> Error {
        match err {
            update_project::Error::NotFound => Error::NotFound,
            update_project::Error::InsufficientPermissions => Error::InsufficientPermissions,
            update_project::Error::InvalidName => Error::InvalidField("name"),
            update_project::Error::InvalidKanaName => Error::InvalidField("kana_name"),
            update_project::Error::InvalidGroupName => Error::InvalidField("group_name"),
            update_project::Error::InvalidKanaGroupName => Error::InvalidField("kana_group_name"),
            update_project::Error::InvalidDescription => Error::InvalidField("description"),
            update_project::Error::DuplicatedAttributes => Error::InvalidField("attributes"),
        }
    }
}

pub async fn handler(app: Login<App>, request: Request) -> HandlerResult<Response, Error> {
    let input = update_project::Input {
        id: request.id.into_use_case(),
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
    let project = update_project::run(&app, input).await?;
    let project = Project::from_use_case(project);
    Ok(Response { project })
}
