use crate::app::Context;
use crate::handler::model::file::File;
use crate::handler::model::file_sharing::FileSharingId;
use crate::handler::model::registration_form_answer::RegistrationFormAnswerId;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::get_registration_form_answer_shared_file;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub answer_id: RegistrationFormAnswerId,
    pub sharing_id: FileSharingId,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub file: File,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
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

impl From<get_registration_form_answer_shared_file::Error> for Error {
    fn from(err: get_registration_form_answer_shared_file::Error) -> Error {
        match err {
            get_registration_form_answer_shared_file::Error::RegistrationFormAnswerNotFound => {
                Error::RegistrationFormAnswerNotFound
            }
            get_registration_form_answer_shared_file::Error::FileSharingNotFound => {
                Error::FileSharingNotFound
            }
            get_registration_form_answer_shared_file::Error::InvalidSharing => {
                Error::InvalidFileSharing
            }
            get_registration_form_answer_shared_file::Error::InsufficientPermissions => {
                Error::InsufficientPermissions
            }
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(ctx: Login<Context>, request: Request) -> HandlerResult<Response, Error> {
    let input = get_registration_form_answer_shared_file::Input {
        answer_id: request.answer_id.into_use_case(),
        sharing_id: request.sharing_id.into_use_case(),
    };
    let file = get_registration_form_answer_shared_file::run(&ctx, input).await?;
    let file = File::from_use_case(file);
    Ok(Response { file })
}
