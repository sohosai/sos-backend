use crate::app::Context;
use crate::handler::model::date_time::DateTime;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use sos21_use_case::get_project_creation_availability;
use sos21_use_case::model::project_creation_availability::ProjectCreationAvailability;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub timestamp: DateTime,
    pub general: bool,
    pub cooking_requiring_preparation_area: bool,
    pub cooking: bool,
    pub food: bool,
    pub stage: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(ctx: Context, _request: Request) -> HandlerResult<Response, Error> {
    let ProjectCreationAvailability {
        timestamp,
        general,
        cooking_requiring_preparation_area,
        cooking,
        food,
        stage,
    } = get_project_creation_availability::run(&ctx);

    Ok(Response {
        timestamp: DateTime::from_use_case(timestamp),
        general,
        cooking_requiring_preparation_area,
        cooking,
        food,
        stage,
    })
}
