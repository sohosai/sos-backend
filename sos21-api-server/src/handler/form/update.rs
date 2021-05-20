use crate::app::Context;
use crate::handler::model::date_time::DateTime;
use crate::handler::model::form::{Form, FormCondition, FormId, FormItem, FormItemId};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::{interface, update_form};
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub id: FormId,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub starts_at: Option<DateTime>,
    #[serde(default)]
    pub ends_at: Option<DateTime>,
    #[serde(default)]
    pub items: Option<Vec<FormItem>>,
    #[serde(default)]
    pub condition: Option<FormCondition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub form: Form,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    FormNotFound,
    InvalidField { field: &'static str },
    InvalidFormItem { id: FormItemId },
    InvalidFormPeriod,
    TooEarlyFormPeriodStart,
    AlreadyStartedForm,
    DuplicatedFormItemId { id: FormItemId },
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::FormNotFound => StatusCode::NOT_FOUND,
            Error::InvalidField { .. } => StatusCode::BAD_REQUEST,
            Error::InvalidFormItem { .. } => StatusCode::BAD_REQUEST,
            Error::InvalidFormPeriod => StatusCode::BAD_REQUEST,
            Error::TooEarlyFormPeriodStart => StatusCode::CONFLICT,
            Error::AlreadyStartedForm => StatusCode::CONFLICT,
            Error::DuplicatedFormItemId { .. } => StatusCode::BAD_REQUEST,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<update_form::Error> for Error {
    fn from(err: update_form::Error) -> Error {
        match err {
            update_form::Error::NotFound => Error::FormNotFound,
            update_form::Error::InvalidName => Error::InvalidField { field: "name" },
            update_form::Error::InvalidDescription => Error::InvalidField {
                field: "description",
            },
            update_form::Error::InvalidItems(err) => match err {
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
            update_form::Error::InvalidCondition(_) => Error::InvalidField { field: "condition" },
            update_form::Error::InvalidPeriod => Error::InvalidFormPeriod,
            update_form::Error::TooEarlyPeriodStart => Error::TooEarlyFormPeriodStart,
            update_form::Error::AlreadyStarted => Error::AlreadyStartedForm,
            update_form::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = update_form::Input {
        id: request.id.into_use_case(),
        name: request.name,
        description: request.description,
        starts_at: request.starts_at.map(DateTime::into_use_case),
        ends_at: request.ends_at.map(DateTime::into_use_case),
        items: request
            .items
            .map(|items| items.into_iter().map(FormItem::into_use_case).collect()),
        condition: request.condition.map(FormCondition::into_use_case),
    };
    let form = update_form::run(&ctx, input).await?;
    let form = Form::from_use_case(form);
    Ok(Response { form })
}
