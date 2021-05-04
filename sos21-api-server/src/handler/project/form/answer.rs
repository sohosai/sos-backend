use crate::app::Context;
use crate::handler::model::{
    form::{FormId, FormItemId},
    form_answer::{item::RequestFormAnswerItem, FormAnswer},
    project::ProjectId,
};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::{answer_form, interface};
use warp::http::StatusCode;

pub mod file_sharing;

pub mod get;
pub use get::handler as get;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub project_id: ProjectId,
    pub form_id: FormId,
    pub items: Vec<RequestFormAnswerItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub answer: FormAnswer,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::CREATED
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    FormNotFound,
    ProjectNotFound,
    OutOfAnswerPeriod,
    AlreadyAnsweredForm,
    NoFormItems,
    TooManyFormItems,
    InvalidFormItem {
        id: FormItemId,
    },
    MismatchedFormItemsLength,
    MismatchedFormItemId {
        expected: FormItemId,
        got: FormItemId,
    },
    InvalidFormAnswer {
        id: FormItemId,
    },
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::FormNotFound => StatusCode::NOT_FOUND,
            Error::ProjectNotFound => StatusCode::NOT_FOUND,
            Error::OutOfAnswerPeriod => StatusCode::BAD_REQUEST,
            Error::AlreadyAnsweredForm => StatusCode::CONFLICT,
            Error::NoFormItems => StatusCode::BAD_REQUEST,
            Error::TooManyFormItems => StatusCode::BAD_REQUEST,
            Error::InvalidFormItem { .. } => StatusCode::BAD_REQUEST,
            Error::MismatchedFormItemsLength => StatusCode::BAD_REQUEST,
            Error::MismatchedFormItemId { .. } => StatusCode::BAD_REQUEST,
            Error::InvalidFormAnswer { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<answer_form::Error> for Error {
    fn from(err: answer_form::Error) -> Error {
        match err {
            answer_form::Error::FormNotFound => Error::FormNotFound,
            answer_form::Error::ProjectNotFound => Error::ProjectNotFound,
            answer_form::Error::OutOfAnswerPeriod => Error::OutOfAnswerPeriod,
            answer_form::Error::AlreadyAnswered => Error::AlreadyAnsweredForm,
            answer_form::Error::InvalidItems(err) => match err {
                interface::form_answer::FormAnswerItemsError::NoItems => Error::NoFormItems,
                interface::form_answer::FormAnswerItemsError::TooManyItems => {
                    Error::TooManyFormItems
                }
                // TODO: break down invalid item errors
                interface::form_answer::FormAnswerItemsError::InvalidItem(id, _) => {
                    Error::InvalidFormItem {
                        id: FormItemId::from_use_case(id),
                    }
                }
            },
            answer_form::Error::InvalidAnswer(err) => match err {
                interface::form::CheckAnswerError::MismatchedItemsLength => {
                    Error::MismatchedFormItemsLength
                }
                interface::form::CheckAnswerError::MismatchedItemId { expected, got } => {
                    Error::MismatchedFormItemId {
                        expected: FormItemId::from_use_case(expected),
                        got: FormItemId::from_use_case(got),
                    }
                }
                // TODO: break down invalid answer errors
                interface::form::CheckAnswerError::InvalidAnswerItem { item_id, .. } => {
                    Error::InvalidFormAnswer {
                        id: FormItemId::from_use_case(item_id),
                    }
                }
            },
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = answer_form::Input {
        project_id: request.project_id.into_use_case(),
        form_id: request.form_id.into_use_case(),
        items: request
            .items
            .into_iter()
            .map(RequestFormAnswerItem::into_use_case)
            .collect(),
    };
    let answer = answer_form::run(&ctx, input).await?;
    let answer = FormAnswer::from_use_case(answer);
    Ok(Response { answer })
}
