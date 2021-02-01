use crate::app::Context;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::export_users;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    #[serde(default)]
    pub field_id: Option<String>,
    #[serde(default)]
    pub field_created_at: Option<String>,
    #[serde(default)]
    pub field_first_name: Option<String>,
    #[serde(default)]
    pub field_last_name: Option<String>,
    #[serde(default)]
    pub field_full_name: Option<String>,
    #[serde(default)]
    pub field_kana_first_name: Option<String>,
    #[serde(default)]
    pub field_kana_last_name: Option<String>,
    #[serde(default)]
    pub field_kana_full_name: Option<String>,
    #[serde(default)]
    pub field_email: Option<String>,
    #[serde(default)]
    pub field_phone_number: Option<String>,
    #[serde(default)]
    pub field_affiliation: Option<String>,
    #[serde(default)]
    pub field_role: Option<String>,
    pub role_administrator: String,
    pub role_committee_operator: String,
    pub role_committee: String,
    pub role_general: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    InsufficientPermissions,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

impl From<export_users::Error> for Error {
    fn from(err: export_users::Error) -> Error {
        match err {
            export_users::Error::InsufficientPermissions => Error::InsufficientPermissions,
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
            field_id,
            field_created_at,
            field_first_name,
            field_last_name,
            field_full_name,
            field_kana_first_name,
            field_kana_last_name,
            field_kana_full_name,
            field_email,
            field_phone_number,
            field_affiliation,
            field_role,
            role_administrator,
            role_committee_operator,
            role_committee,
            role_general,
        } = request;
        let field_names = export_users::InputFieldNames {
            id: field_id,
            created_at: field_created_at,
            first_name: field_first_name,
            last_name: field_last_name,
            full_name: field_full_name,
            kana_first_name: field_kana_first_name,
            kana_last_name: field_kana_last_name,
            kana_full_name: field_kana_full_name,
            email: field_email,
            phone_number: field_phone_number,
            affiliation: field_affiliation,
            role: field_role,
        };
        let role_names = export_users::InputRoleNames {
            administrator: role_administrator,
            committee_operator: role_committee_operator,
            committee: role_committee,
            general: role_general,
        };
        export_users::Input {
            field_names,
            role_names,
        }
    };

    let csv = export_users::run(&ctx, input).await?;
    Ok(warp::reply::with_status(
        warp::reply::with_header(csv, warp::http::header::CONTENT_TYPE, "text/csv"),
        StatusCode::OK,
    ))
}
