use crate::app::App;
use crate::handler::{HandlerResponse, HandlerResult};

use rusoto_s3::S3;
use serde::{Deserialize, Serialize};
use warp::http::StatusCode;

pub mod liveness;
pub use liveness::handler as liveness;
pub mod database;
pub use database::handler as database;
pub mod s3;
pub use s3::handler as s3;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub database: bool,
    pub s3: bool,
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
    let database = sos21_database::query::is_healthy(&mut app.connection().await?).await?;

    let s3 = app
        .s3_client()
        .head_bucket(rusoto_s3::HeadBucketRequest {
            bucket: app.config().s3_object_bucket.clone(),
            ..Default::default()
        })
        .await
        .is_ok();

    Ok(Response { database, s3 })
}
