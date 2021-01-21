use serde::Serialize;
use warp::http::StatusCode;

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub ok: bool,
}

impl crate::filter::Response for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

pub async fn liveness() -> anyhow::Result<Response> {
    Ok(Response { ok: true })
}
