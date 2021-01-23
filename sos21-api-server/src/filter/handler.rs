use std::future::Future;

use crate::handler::{HandlerError, HandlerResponse, HandlerResult};

use warp::{
    http::StatusCode,
    {reject::Rejection, reply::Reply},
};

/// Type erased [`HandlerError`].
#[derive(Debug)]
pub enum ErasedHandlerError {
    Client {
        status_code: StatusCode,
        info: serde_json::Value,
    },
    Server(anyhow::Error),
}

impl<E> From<HandlerError<E>> for ErasedHandlerError
where
    E: HandlerResponse,
{
    fn from(err: HandlerError<E>) -> Self {
        match err {
            HandlerError::Client(err) => {
                let status_code = err.status_code();
                let info = match serde_json::to_value(&err) {
                    Ok(info) => info,
                    Err(e) => return ErasedHandlerError::Server(e.into()),
                };
                ErasedHandlerError::Client { status_code, info }
            }
            HandlerError::Server(err) => ErasedHandlerError::Server(err),
        }
    }
}

impl warp::reject::Reject for ErasedHandlerError {}

pub async fn run_handler<F, R, E>(fut: F) -> Result<impl Reply, Rejection>
where
    F: Future<Output = HandlerResult<R, E>>,
    R: HandlerResponse,
    E: HandlerResponse,
{
    fut.await
        .map_err(|error| warp::reject::custom(ErasedHandlerError::from(error)))
        .map(|response| {
            let code = response.status_code();
            debug_assert!(code.is_success());
            warp::reply::with_status(warp::reply::json(&response), code)
        })
}
