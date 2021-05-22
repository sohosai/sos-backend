use crate::app::Context;
use crate::handler::model::{
    form::FormItemId, form_answer::item::RequestFormAnswerItem, pending_project::PendingProjectId,
    registration_form::RegistrationFormId, registration_form_answer::RegistrationFormAnswer,
};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::{answer_registration_form, interface};
use warp::http::StatusCode;

pub mod get;
pub use get::handler as get;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub pending_project_id: PendingProjectId,
    pub registration_form_id: RegistrationFormId,
    pub items: Vec<RequestFormAnswerItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub answer: RegistrationFormAnswer,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::CREATED
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    RegistrationFormNotFound,
    PendingProjectNotFound,
    AlreadyAnsweredRegistrationForm,
    OutOfProjectCreationPeriod,
    NoFormAnswerItems,
    TooManyFormAnswerItems,
    InvalidFormAnswerItem {
        id: FormItemId,
    },
    MismatchedFormItemsLength,
    MismatchedFormItemId {
        expected: FormItemId,
        got: FormItemId,
    },
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::RegistrationFormNotFound => StatusCode::NOT_FOUND,
            Error::PendingProjectNotFound => StatusCode::NOT_FOUND,
            Error::AlreadyAnsweredRegistrationForm => StatusCode::CONFLICT,
            Error::OutOfProjectCreationPeriod => StatusCode::CONFLICT,
            Error::NoFormAnswerItems => StatusCode::BAD_REQUEST,
            Error::TooManyFormAnswerItems => StatusCode::BAD_REQUEST,
            Error::InvalidFormAnswerItem { .. } => StatusCode::BAD_REQUEST,
            Error::MismatchedFormItemsLength => StatusCode::BAD_REQUEST,
            Error::MismatchedFormItemId { .. } => StatusCode::BAD_REQUEST,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<answer_registration_form::Error> for Error {
    fn from(err: answer_registration_form::Error) -> Error {
        match err {
            answer_registration_form::Error::RegistrationFormNotFound => {
                Error::RegistrationFormNotFound
            }
            answer_registration_form::Error::PendingProjectNotFound => {
                Error::PendingProjectNotFound
            }
            answer_registration_form::Error::AlreadyAnswered => {
                Error::AlreadyAnsweredRegistrationForm
            }
            answer_registration_form::Error::OutOfProjectCreationPeriod => {
                Error::OutOfProjectCreationPeriod
            }
            answer_registration_form::Error::InvalidItems(err) => match err {
                interface::form_answer::FormAnswerItemsError::NoItems => Error::NoFormAnswerItems,
                interface::form_answer::FormAnswerItemsError::TooManyItems => {
                    Error::TooManyFormAnswerItems
                }
                // TODO: break down invalid item errors
                interface::form_answer::FormAnswerItemsError::InvalidItem(id, _) => {
                    Error::InvalidFormAnswerItem {
                        id: FormItemId::from_use_case(id),
                    }
                }
            },
            answer_registration_form::Error::InvalidAnswer(err) => match err {
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
                    Error::InvalidFormAnswerItem {
                        id: FormItemId::from_use_case(item_id),
                    }
                }
            },
            answer_registration_form::Error::InsufficientPermissions => {
                Error::InsufficientPermissions
            }
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = answer_registration_form::Input {
        pending_project_id: request.pending_project_id.into_use_case(),
        registration_form_id: request.registration_form_id.into_use_case(),
        items: request
            .items
            .into_iter()
            .map(RequestFormAnswerItem::into_use_case)
            .collect(),
    };
    let answer = answer_registration_form::run(&ctx, input).await?;
    let answer = RegistrationFormAnswer::from_use_case(answer);
    Ok(Response { answer })
}
