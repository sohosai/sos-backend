use crate::app::Context;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_domain::context::Login;
use sos21_use_case::export_projects;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    #[serde(default)]
    pub field_id: Option<String>,
    #[serde(default)]
    pub field_code: Option<String>,
    #[serde(default)]
    pub field_created_at: Option<String>,
    #[serde(default)]
    pub field_updated_at: Option<String>,
    #[serde(default)]
    pub field_owner_id: Option<String>,
    #[serde(default)]
    pub field_owner_first_name: Option<String>,
    #[serde(default)]
    pub field_owner_last_name: Option<String>,
    #[serde(default)]
    pub field_owner_full_name: Option<String>,
    #[serde(default)]
    pub field_owner_kana_first_name: Option<String>,
    #[serde(default)]
    pub field_owner_kana_last_name: Option<String>,
    #[serde(default)]
    pub field_owner_kana_full_name: Option<String>,
    #[serde(default)]
    pub field_subowner_id: Option<String>,
    #[serde(default)]
    pub field_subowner_first_name: Option<String>,
    #[serde(default)]
    pub field_subowner_last_name: Option<String>,
    #[serde(default)]
    pub field_subowner_full_name: Option<String>,
    #[serde(default)]
    pub field_subowner_kana_first_name: Option<String>,
    #[serde(default)]
    pub field_subowner_kana_last_name: Option<String>,
    #[serde(default)]
    pub field_subowner_kana_full_name: Option<String>,
    #[serde(default)]
    pub field_name: Option<String>,
    #[serde(default)]
    pub field_kana_name: Option<String>,
    #[serde(default)]
    pub field_group_name: Option<String>,
    #[serde(default)]
    pub field_kana_group_name: Option<String>,
    #[serde(default)]
    pub field_description: Option<String>,
    #[serde(default)]
    pub field_category: Option<String>,
    #[serde(default)]
    pub field_attribute_academic: Option<String>,
    #[serde(default)]
    pub field_attribute_artistic: Option<String>,
    #[serde(default)]
    pub field_attribute_committee: Option<String>,
    #[serde(default)]
    pub field_attribute_outdoor: Option<String>,
    #[serde(default)]
    pub field_attribute_indoor: Option<String>,
    pub category_general: String,
    pub category_cooking_requiring_preparation_area: String,
    pub category_cooking: String,
    pub category_food: String,
    pub category_stage: String,
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

impl From<export_projects::Error> for Error {
    fn from(err: export_projects::Error) -> Error {
        match err {
            export_projects::Error::InsufficientPermissions => Error::InsufficientPermissions,
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
            field_id,
            field_code,
            field_created_at,
            field_updated_at,
            field_owner_id,
            field_owner_first_name,
            field_owner_last_name,
            field_owner_full_name,
            field_owner_kana_first_name,
            field_owner_kana_last_name,
            field_owner_kana_full_name,
            field_subowner_id,
            field_subowner_first_name,
            field_subowner_last_name,
            field_subowner_full_name,
            field_subowner_kana_first_name,
            field_subowner_kana_last_name,
            field_subowner_kana_full_name,
            field_name,
            field_kana_name,
            field_group_name,
            field_kana_group_name,
            field_description,
            field_category,
            field_attribute_academic,
            field_attribute_artistic,
            field_attribute_committee,
            field_attribute_outdoor,
            field_attribute_indoor,
            category_general,
            category_cooking_requiring_preparation_area,
            category_cooking,
            category_food,
            category_stage,
        } = request;
        let field_names = export_projects::InputFieldNames {
            id: field_id,
            code: field_code,
            created_at: field_created_at,
            updated_at: field_updated_at,
            owner_id: field_owner_id,
            owner_first_name: field_owner_first_name,
            owner_last_name: field_owner_last_name,
            owner_full_name: field_owner_full_name,
            owner_kana_first_name: field_owner_kana_first_name,
            owner_kana_last_name: field_owner_kana_last_name,
            owner_kana_full_name: field_owner_kana_full_name,
            subowner_id: field_subowner_id,
            subowner_first_name: field_subowner_first_name,
            subowner_last_name: field_subowner_last_name,
            subowner_full_name: field_subowner_full_name,
            subowner_kana_first_name: field_subowner_kana_first_name,
            subowner_kana_last_name: field_subowner_kana_last_name,
            subowner_kana_full_name: field_subowner_kana_full_name,
            name: field_name,
            kana_name: field_kana_name,
            group_name: field_group_name,
            kana_group_name: field_kana_group_name,
            description: field_description,
            category: field_category,
            attribute_academic: field_attribute_academic,
            attribute_artistic: field_attribute_artistic,
            attribute_committee: field_attribute_committee,
            attribute_outdoor: field_attribute_outdoor,
            attribute_indoor: field_attribute_indoor,
        };
        let category_names = export_projects::InputCategoryNames {
            general: category_general,
            cooking_requiring_preparation_area: category_cooking_requiring_preparation_area,
            cooking: category_cooking,
            food: category_food,
            stage: category_stage,
        };
        export_projects::Input {
            field_names,
            category_names,
        }
    };
    let csv = export_projects::run(&ctx, input).await?;
    Ok(warp::reply::with_status(
        warp::reply::with_header(csv, warp::http::header::CONTENT_TYPE, "text/csv"),
        StatusCode::OK,
    ))
}
