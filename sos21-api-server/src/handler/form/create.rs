use crate::app::Context;
use crate::handler::model::form::{Form, FormCondition, FormItem, FormItemId};
use crate::handler::{HandlerResponse, HandlerResult};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::create_form;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub name: String,
    pub description: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub items: Vec<FormItem>,
    pub condition: FormCondition,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub form: Form,
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
    InvalidFormPeriod,
    DuplicatedFormItemId { id: FormItemId },
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidField { .. } => StatusCode::BAD_REQUEST,
            Error::InvalidFormItem { .. } => StatusCode::BAD_REQUEST,
            Error::InvalidFormPeriod => StatusCode::BAD_REQUEST,
            Error::DuplicatedFormItemId { .. } => StatusCode::BAD_REQUEST,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<create_form::Error> for Error {
    fn from(err: create_form::Error) -> Error {
        match err {
            create_form::Error::InvalidName => Error::InvalidField { field: "name" },
            create_form::Error::InvalidDescription => Error::InvalidField {
                field: "description",
            },
            create_form::Error::NoItems => Error::InvalidField { field: "items" },
            create_form::Error::TooManyItems => Error::InvalidField { field: "items" },
            // TODO: break down invalid item errors
            create_form::Error::InvalidItem(id, _) => Error::InvalidFormItem {
                id: FormItemId::from_use_case(id),
            },
            create_form::Error::InvalidCondition(_) => Error::InvalidField { field: "condition" },
            create_form::Error::InvalidPeriod => Error::InvalidFormPeriod,
            create_form::Error::DuplicatedItemId(id) => Error::DuplicatedFormItemId {
                id: FormItemId::from_use_case(id),
            },
            create_form::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = create_form::Input {
        name: request.name,
        description: request.description,
        starts_at: request.starts_at,
        ends_at: request.ends_at,
        items: request
            .items
            .into_iter()
            .map(FormItem::into_use_case)
            .collect(),
        condition: request.condition.into_use_case(),
    };
    let form = create_form::run(&ctx, input).await?;
    let form = Form::from_use_case(form);
    Ok(Response { form })
}
