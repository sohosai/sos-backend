use crate::app::App;
use crate::handler::{HandlerError, HandlerResponse, HandlerResult};

use anyhow::Context as _;
use rusoto_core::RusotoError;
use rusoto_s3::S3;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use warp::http::StatusCode;

#[derive(Debug, Clone, Deserialize)]
pub struct Request {}

#[derive(Debug, Clone, Serialize)]
pub struct Response;

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

#[derive(Debug, Error)]
pub enum UnavailableError {
    #[error("Unable to reach the s3 bucket")]
    UnreachableS3Bucket(#[source] rusoto_s3::HeadBucketError),
    #[error("The database state is not healthy")]
    UnhealthyDatabase,
}

async fn check_s3(app: &App) -> HandlerResult<(), Error> {
    let head_request = rusoto_s3::HeadBucketRequest {
        bucket: app.config().s3_object_bucket.clone(),
        ..Default::default()
    };

    match app.s3_client().head_bucket(head_request).await {
        Err(RusotoError::Service(err)) => Err(HandlerError::ServiceUnavailable(
            UnavailableError::UnreachableS3Bucket(err).into(),
        )),
        result => Ok(result.context("Failed to dispatch HeadBucketRequest")?),
    }
}

async fn check_database(app: &App) -> HandlerResult<(), Error> {
    let mut conn = app.connection().await?;
    if !sos21_database::query::is_healthy(&mut conn).await? {
        return Err(HandlerError::ServiceUnavailable(
            UnavailableError::UnhealthyDatabase.into(),
        ));
    }

    Ok(())
}

#[macro_rules_attribute::macro_rules_attribute(handler!)]
pub async fn handler(app: App, _request: Request) -> HandlerResult<Response, Error> {
    check_s3(&app).await?;
    check_database(&app).await?;

    Ok(Response)
}
