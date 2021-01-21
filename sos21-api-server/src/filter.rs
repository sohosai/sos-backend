use std::convert::Infallible;
use std::future::Future;

use serde::Serialize;
use warp::{
    http::StatusCode,
    Filter,
    {reject::Rejection, reply::Reply},
};

pub trait Response: Serialize {
    fn status_code(&self) -> StatusCode;
}

async fn handle(
    fut: impl Future<Output = Result<impl Response, anyhow::Error>>,
) -> Result<impl Reply, Infallible> {
    Ok(match fut.await {
        Ok(x) => warp::reply::with_status(warp::reply::json(&x), x.status_code()).into_response(),
        Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    })
}

pub fn app() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    use crate::handler;
    warp::path("health").and({
        use handler::health;
        warp::path("liveness")
            .map(health::liveness)
            .and_then(handle)
    })
}
