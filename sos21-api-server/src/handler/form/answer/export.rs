use crate::app::Context;
use crate::handler::model::form::FormId;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::export_form_answers;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub form_id: FormId,
    #[serde(default)]
    pub field_id: Option<String>,
    #[serde(default)]
    pub field_created_at: Option<String>,
    #[serde(default)]
    pub field_project_id: Option<String>,
    #[serde(default)]
    pub field_author_id: Option<String>,
    pub checkbox_checked: String,
    pub checkbox_unchecked: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    FormNotFound,
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::FormNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<export_form_answers::Error> for Error {
    fn from(err: export_form_answers::Error) -> Error {
        match err {
            export_form_answers::Error::FormNotFound => Error::FormNotFound,
            export_form_answers::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[apply_macro::apply(raw_response_handler)]
pub async fn handler(
    ctx: Login<Context>,
    request: Request,
) -> HandlerResult<impl warp::Reply, Error> {
    let input = {
        let Request {
            form_id,
            field_id,
            field_created_at,
            field_project_id,
            field_author_id,
            checkbox_checked,
            checkbox_unchecked,
        } = request;
        let field_names = export_form_answers::InputFieldNames {
            id: field_id,
            created_at: field_created_at,
            project_id: field_project_id,
            author_id: field_author_id,
        };
        let checkbox_names = export_form_answers::InputCheckboxNames {
            checked: checkbox_checked,
            unchecked: checkbox_unchecked,
        };
        export_form_answers::Input {
            form_id: form_id.into_use_case(),
            field_names,
            checkbox_names,
        }
    };

    let csv = export_form_answers::run(&ctx, input).await?;
    Ok(warp::reply::with_status(
        warp::reply::with_header(csv, warp::http::header::CONTENT_TYPE, "text/csv"),
        StatusCode::OK,
    ))
}
