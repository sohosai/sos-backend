use crate::app::Context;
use crate::handler::model::project::{Project, ProjectAttribute, ProjectCategory};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::create_project;
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
    pub project: Project,
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
    TooManyProjects,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidField { .. } => StatusCode::BAD_REQUEST,
            Error::DuplicatedProjectAttributes => StatusCode::BAD_REQUEST,
            Error::TooManyProjects => StatusCode::CONFLICT,
        }
    }
}

impl From<create_project::Error> for Error {
    fn from(err: create_project::Error) -> Error {
        match err {
            create_project::Error::InvalidName => Error::InvalidField { field: "name" },
            create_project::Error::InvalidKanaName => Error::InvalidField { field: "kana_name" },
            create_project::Error::InvalidGroupName => Error::InvalidField {
                field: "group_name",
            },
            create_project::Error::InvalidKanaGroupName => Error::InvalidField {
                field: "kana_group_name",
            },
            create_project::Error::InvalidDescription => Error::InvalidField {
                field: "description",
            },
            create_project::Error::DuplicatedAttributes => Error::DuplicatedProjectAttributes,
            create_project::Error::TooManyProjects => Error::TooManyProjects,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = create_project::Input {
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
    let project = create_project::run(&ctx, input).await?;
    let project = Project::from_use_case(project);
    Ok(Response { project })
}
