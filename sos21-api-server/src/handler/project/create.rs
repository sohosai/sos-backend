use crate::app::Context;
use crate::handler::model::project::{Project, ProjectAttribute, ProjectCategory};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain_context::Login;
use sos21_use_case::create_project;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub display_id: String,
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
pub enum Error {
    InvalidField(&'static str),
    DuplicatedAttributes,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidField(_) => StatusCode::BAD_REQUEST,
            Error::DuplicatedAttributes => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<create_project::Error> for Error {
    fn from(err: create_project::Error) -> Error {
        match err {
            create_project::Error::InvalidDisplayId => Error::InvalidField("display_id"),
            create_project::Error::InvalidName => Error::InvalidField("name"),
            create_project::Error::InvalidKanaName => Error::InvalidField("kana_name"),
            create_project::Error::InvalidGroupName => Error::InvalidField("group_name"),
            create_project::Error::InvalidKanaGroupName => Error::InvalidField("kana_group_name"),
            create_project::Error::InvalidDescription => Error::InvalidField("description"),
            create_project::Error::DuplicatedAttributes => Error::DuplicatedAttributes,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = create_project::Input {
        display_id: request.display_id,
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
