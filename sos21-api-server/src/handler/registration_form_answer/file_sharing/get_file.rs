use crate::app::Context;
use crate::handler::model::file::FileObject;
use crate::handler::model::file_sharing::FileSharingId;
use crate::handler::model::registration_form_answer::RegistrationFormAnswerId;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_registration_form_answer_shared_file_object;
use warp::{http::StatusCode, reply};

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub answer_id: RegistrationFormAnswerId,
    pub sharing_id: FileSharingId,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    RegistrationFormAnswerNotFound,
    FileSharingNotFound,
    InvalidFileSharing,
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::RegistrationFormAnswerNotFound => StatusCode::NOT_FOUND,
            Error::FileSharingNotFound => StatusCode::NOT_FOUND,
            Error::InvalidFileSharing => StatusCode::FORBIDDEN,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<get_registration_form_answer_shared_file_object::Error> for Error {
    fn from(err: get_registration_form_answer_shared_file_object::Error) -> Error {
        match err {
            get_registration_form_answer_shared_file_object::Error::FileSharingNotFound => {
                Error::FileSharingNotFound
            }
            get_registration_form_answer_shared_file_object::Error::RegistrationFormAnswerNotFound => {
                Error::RegistrationFormAnswerNotFound
            }
            get_registration_form_answer_shared_file_object::Error::InvalidSharing => Error::InvalidFileSharing,
            get_registration_form_answer_shared_file_object::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[apply_macro::apply(raw_response_handler)]
pub async fn handler(
    ctx: Login<Context>,
    request: Request,
) -> HandlerResult<impl warp::Reply, Error> {
    let input = get_registration_form_answer_shared_file_object::Input {
        answer_id: request.answer_id.into_use_case(),
        sharing_id: request.sharing_id.into_use_case(),
    };
    let file_object = get_registration_form_answer_shared_file_object::run(&ctx, input).await?;
    let file_object = FileObject::from_use_case(file_object);
    let reply = file_object.into_reply();
    let reply = reply::with_status(reply, StatusCode::OK);
    Ok(reply)
}
