use crate::app::Context;
use crate::handler::model::{
    file::FileId,
    file_sharing::FileSharingId,
    form::{
        item::{CheckboxId, RadioId},
        FormId, FormItemId,
    },
    form_answer::{item::GridRadioRowAnswer, FormAnswer},
    project::ProjectId,
};
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::{create_form_answer, interface};
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

#[derive(Debug, Clone, Deserialize)]
pub struct RequestFormAnswerItem {
    pub item_id: FormItemId,
    #[serde(flatten)]
    pub body: Option<RequestFormAnswerItemBody>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "answer")]
pub enum RequestFormAnswerItemBody {
    Text(Option<String>),
    Integer(Option<u64>),
    Checkbox(Vec<CheckboxId>),
    Radio(Option<RadioId>),
    GridRadio(Vec<GridRadioRowAnswer>),
    File(Vec<RequestFormAnswerItemFile>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestFormAnswerItemFile {
    SharingId(FileSharingId),
    FileId(FileId),
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

impl From<create_form_answer::Error> for Error {
    fn from(err: create_form_answer::Error) -> Error {
        match err {
            create_form_answer::Error::FormNotFound => Error::FormNotFound,
            create_form_answer::Error::ProjectNotFound => Error::ProjectNotFound,
            create_form_answer::Error::OutOfAnswerPeriod => Error::OutOfAnswerPeriod,
            create_form_answer::Error::AlreadyAnswered => Error::AlreadyAnsweredForm,
            create_form_answer::Error::InvalidItems(err) => match err {
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
            create_form_answer::Error::InvalidAnswer(err) => match err {
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
    let input = create_form_answer::Input {
        project_id: request.project_id.into_use_case(),
        form_id: request.form_id.into_use_case(),
        items: request
            .items
            .into_iter()
            .map(to_input_form_answer_item)
            .collect(),
    };
    let answer = create_form_answer::run(&ctx, input).await?;
    let answer = FormAnswer::from_use_case(answer);
    Ok(Response { answer })
}

fn to_input_form_answer_item(
    item: RequestFormAnswerItem,
) -> interface::form_answer::InputFormAnswerItem {
    interface::form_answer::InputFormAnswerItem {
        item_id: item.item_id.into_use_case(),
        body: item.body.map(to_input_form_answer_item_body),
    }
}

fn to_input_form_answer_item_body(
    body: RequestFormAnswerItemBody,
) -> interface::form_answer::InputFormAnswerItemBody {
    match body {
        RequestFormAnswerItemBody::Text(answer) => {
            interface::form_answer::InputFormAnswerItemBody::Text(answer)
        }
        RequestFormAnswerItemBody::Integer(answer) => {
            interface::form_answer::InputFormAnswerItemBody::Integer(answer)
        }
        RequestFormAnswerItemBody::Checkbox(answer) => {
            interface::form_answer::InputFormAnswerItemBody::Checkbox(
                answer.into_iter().map(CheckboxId::into_use_case).collect(),
            )
        }
        RequestFormAnswerItemBody::Radio(answer) => {
            interface::form_answer::InputFormAnswerItemBody::Radio(
                answer.map(RadioId::into_use_case),
            )
        }
        RequestFormAnswerItemBody::GridRadio(answer) => {
            interface::form_answer::InputFormAnswerItemBody::GridRadio(
                answer
                    .into_iter()
                    .map(GridRadioRowAnswer::into_use_case)
                    .collect(),
            )
        }
        RequestFormAnswerItemBody::File(answer) => {
            interface::form_answer::InputFormAnswerItemBody::File(
                answer
                    .into_iter()
                    .map(to_input_form_answer_item_file)
                    .collect(),
            )
        }
    }
}

fn to_input_form_answer_item_file(
    file: RequestFormAnswerItemFile,
) -> interface::form_answer::InputFormAnswerItemFile {
    match file {
        RequestFormAnswerItemFile::FileId(file_id) => {
            interface::form_answer::InputFormAnswerItemFile::File(file_id.into_use_case())
        }
        RequestFormAnswerItemFile::SharingId(sharing_id) => {
            interface::form_answer::InputFormAnswerItemFile::Sharing(sharing_id.into_use_case())
        }
    }
}
