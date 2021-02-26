use crate::app::Context;
use crate::handler::model::{
    form::{FormId, FormItemId},
    form_answer::{FormAnswer, FormAnswerItem},
    project::ProjectId,
};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::create_form_answer;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub project_id: ProjectId,
    pub form_id: FormId,
    pub items: Vec<FormAnswerItem>,
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
            Error::AlreadyAnsweredForm => StatusCode::BAD_REQUEST,
            Error::NoFormItems => StatusCode::BAD_REQUEST,
            Error::TooManyFormItems => StatusCode::BAD_REQUEST,
            Error::InvalidFormItem { .. } => StatusCode::BAD_REQUEST,
            Error::MismatchedFormItemsLength => StatusCode::BAD_REQUEST,
            Error::MismatchedFormItemId { .. } => StatusCode::BAD_REQUEST,
            Error::InvalidFormAnswer { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<create_form_answer::Error> for Error {
    fn from(err: create_form_answer::Error) -> Error {
        match err {
            create_form_answer::Error::FormNotFound => Error::FormNotFound,
            create_form_answer::Error::ProjectNotFound => Error::ProjectNotFound,
            create_form_answer::Error::OutOfAnswerPeriod => Error::OutOfAnswerPeriod,
            create_form_answer::Error::AlreadyAnswered => Error::AlreadyAnsweredForm,
            create_form_answer::Error::NoItems => Error::NoFormItems,
            create_form_answer::Error::TooManyItems => Error::TooManyFormItems,
            // TODO: break down invalid item errors
            create_form_answer::Error::InvalidItem(id, _) => Error::InvalidFormItem {
                id: FormItemId::from_use_case(id),
            },
            create_form_answer::Error::MismatchedItemsLength => Error::MismatchedFormItemsLength,
            create_form_answer::Error::MismatchedItemId { expected, got } => {
                Error::MismatchedFormItemId {
                    expected: FormItemId::from_use_case(expected),
                    got: FormItemId::from_use_case(got),
                }
            }
            // TODO: break down invalid answer errors
            create_form_answer::Error::InvalidAnswer(id, _) => Error::InvalidFormAnswer {
                id: FormItemId::from_use_case(id),
            },
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = create_form_answer::Input {
        project_id: request.project_id.into_use_case(),
        form_id: request.form_id.into_use_case(),
        items: request
            .items
            .into_iter()
            .map(FormAnswerItem::into_use_case)
            .collect(),
    };
    let answer = create_form_answer::run(&ctx, input).await?;
    let answer = FormAnswer::from_use_case(answer);
    Ok(Response { answer })
}
