use crate::app::Context;
use crate::handler::model::form::{FormItem, FormItemId};
use crate::handler::model::project_query::ProjectQuery;
use crate::handler::model::registration_form::RegistrationForm;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::{create_registration_form, interface};
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub name: String,
    pub description: String,
    pub items: Vec<FormItem>,
    pub query: ProjectQuery,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub registration_form: RegistrationForm,
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
    InvalidFormItem { id: FormItemId },
    DuplicatedFormItemId { id: FormItemId },
    InsufficientPermissions,
    AlreadyStartedProjectCreationPeriod,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidField { .. } => StatusCode::BAD_REQUEST,
            Error::InvalidFormItem { .. } => StatusCode::BAD_REQUEST,
            Error::DuplicatedFormItemId { .. } => StatusCode::BAD_REQUEST,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
            Error::AlreadyStartedProjectCreationPeriod => StatusCode::CONFLICT,
        }
    }
}

impl From<create_registration_form::Error> for Error {
    fn from(err: create_registration_form::Error) -> Error {
        match err {
            create_registration_form::Error::InvalidName => Error::InvalidField { field: "name" },
            create_registration_form::Error::InvalidDescription => Error::InvalidField {
                field: "description",
            },
            create_registration_form::Error::InvalidItems(err) => match err {
                interface::form::FormItemsError::NoItems => Error::InvalidField { field: "items" },
                interface::form::FormItemsError::TooManyItems => {
                    Error::InvalidField { field: "items" }
                }
                // TODO: break down invalid item errors
                interface::form::FormItemsError::InvalidItem(id, _) => Error::InvalidFormItem {
                    id: FormItemId::from_use_case(id),
                },
                interface::form::FormItemsError::DuplicatedItemId(id) => {
                    Error::DuplicatedFormItemId {
                        id: FormItemId::from_use_case(id),
                    }
                }
            },
            create_registration_form::Error::InvalidQuery(_) => {
                Error::InvalidField { field: "query" }
            }
            create_registration_form::Error::InsufficientPermissions => {
                Error::InsufficientPermissions
            }
            create_registration_form::Error::AlreadyStartedProjectCreationPeriod => {
                Error::AlreadyStartedProjectCreationPeriod
            }
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = create_registration_form::Input {
        name: request.name,
        description: request.description,
        items: request
            .items
            .into_iter()
            .map(FormItem::into_use_case)
            .collect(),
        query: request.query.into_use_case(),
    };
    let registration_form = create_registration_form::run(&ctx, input).await?;
    let registration_form = RegistrationForm::from_use_case(registration_form);
    Ok(Response { registration_form })
}
