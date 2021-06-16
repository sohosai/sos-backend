use crate::app::Context;
use crate::handler::model::registration_form::RegistrationFormId;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::export_registration_form_answers;
use uritemplate::UriTemplate;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub registration_form_id: RegistrationFormId,
    #[serde(default)]
    pub field_id: Option<String>,
    #[serde(default)]
    pub field_created_at: Option<String>,
    #[serde(default)]
    pub field_project_id: Option<String>,
    #[serde(default)]
    pub field_pending_project_id: Option<String>,
    #[serde(default)]
    pub field_author_id: Option<String>,
    pub file_answer_template: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    RegistrationFormNotFound,
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::RegistrationFormNotFound => StatusCode::NOT_FOUND,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<export_registration_form_answers::Error> for Error {
    fn from(err: export_registration_form_answers::Error) -> Error {
        match err {
            export_registration_form_answers::Error::RegistrationFormNotFound => {
                Error::RegistrationFormNotFound
            }
            export_registration_form_answers::Error::InsufficientPermissions => {
                Error::InsufficientPermissions
            }
        }
    }
}

#[macro_rules_attribute::macro_rules_attribute(raw_response_handler!)]
pub async fn handler(
    ctx: Login<Context>,
    request: Request,
) -> HandlerResult<impl warp::Reply, Error> {
    let input = {
        let Request {
            registration_form_id,
            field_id,
            field_created_at,
            field_project_id,
            field_pending_project_id,
            field_author_id,
            file_answer_template,
        } = request;

        let render_file_answer =
            move |input: export_registration_form_answers::RenderFileAnswerInput| {
                // TODO: Prepare outside the closure to prevent duplication of work
                Ok(UriTemplate::new(&file_answer_template)
                    .set("answer_id", input.answer_id)
                    .set("sharing_ids", input.sharing_ids)
                    .build())
            };

        let field_names = export_registration_form_answers::InputFieldNames {
            id: field_id,
            created_at: field_created_at,
            project_id: field_project_id,
            pending_project_id: field_pending_project_id,
            author_id: field_author_id,
        };
        export_registration_form_answers::Input {
            registration_form_id: registration_form_id.into_use_case(),
            field_names,
            render_file_answer,
        }
    };

    let csv = export_registration_form_answers::run(&ctx, input).await?;
    Ok(warp::reply::with_status(
        warp::reply::with_header(csv, warp::http::header::CONTENT_TYPE, "text/csv"),
        StatusCode::OK,
    ))
}
