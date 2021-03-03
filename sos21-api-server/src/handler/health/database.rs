use crate::app::App;
use crate::handler::{HandlerResponse, HandlerResult};

use serde::{Deserialize, Serialize};
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub ok: bool,
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

#[apply_macro::apply(handler)]
pub async fn handler(app: App, _request: Request) -> HandlerResult<Response, Error> {
    let is_health = sos21_database::query::is_healthy(&mut app.connection().await?).await?;
    Ok(Response { ok: is_health })
}
