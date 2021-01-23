use std::convert::Infallible;

use super::authentication::AuthenticationError;
use super::handler::ErasedHandlerError;

use tracing::{event, Level};
use warp::{
    http::StatusCode,
    {reject::Rejection, reply::Reply},
};

mod model;
use model::{AuthenticationErrorId, Error, ErrorBody, RequestErrorId};

// TODO: Can't we somehow type `Rejection` and detect unhandled rejections statically?
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let error = if err.is_not_found() {
        Error {
            error: ErrorBody::Request {
                id: RequestErrorId::NotFound,
            },
            status: StatusCode::NOT_FOUND,
        }
    } else if let Some(err) = err.find::<ErasedHandlerError>() {
        match err {
            ErasedHandlerError::Server(error) => {
                event!(Level::ERROR, %error, "Unexpected error in handler");
                Error {
                    error: ErrorBody::Internal,
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
            ErasedHandlerError::Client { status_code, info } => Error {
                error: ErrorBody::Api { info: info.clone() },
                status: *status_code,
            },
        }
    } else if let Some(err) = err.find::<AuthenticationError>() {
        match err {
            AuthenticationError::Unauthorized => Error {
                error: ErrorBody::Authentication {
                    id: AuthenticationErrorId::Unauthrized,
                },
                status: StatusCode::UNAUTHORIZED,
            },
            AuthenticationError::InvalidToken => Error {
                error: ErrorBody::Authentication {
                    id: AuthenticationErrorId::InvalidToken,
                },
                status: StatusCode::UNAUTHORIZED,
            },
            AuthenticationError::NoEmail => Error {
                error: ErrorBody::Authentication {
                    id: AuthenticationErrorId::NoEmail,
                },
                status: StatusCode::BAD_REQUEST,
            },
            AuthenticationError::UnverifiedEmail => Error {
                error: ErrorBody::Authentication {
                    id: AuthenticationErrorId::UnverifiedEmail,
                },
                status: StatusCode::FORBIDDEN,
            },
        }
    } else if let Some(_) = err.find::<warp::reject::UnsupportedMediaType>() {
        Error {
            error: ErrorBody::Request {
                id: RequestErrorId::UnsupportedMediaType,
            },
            status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
        }
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        Error {
            error: ErrorBody::Request {
                id: RequestErrorId::MethodNotAllowed,
            },
            status: StatusCode::METHOD_NOT_ALLOWED,
        }
    } else if let Some(_) = err.find::<warp::reject::MissingHeader>() {
        Error {
            error: ErrorBody::Request {
                id: RequestErrorId::MissingHeader,
            },
            status: StatusCode::BAD_REQUEST,
        }
    } else if let Some(_) = err.find::<warp::reject::InvalidHeader>() {
        Error {
            error: ErrorBody::Request {
                id: RequestErrorId::InvalidHeader,
            },
            status: StatusCode::BAD_REQUEST,
        }
    } else if let Some(_) = err.find::<warp::reject::InvalidQuery>() {
        Error {
            error: ErrorBody::Request {
                id: RequestErrorId::InvalidQuery,
            },
            status: StatusCode::BAD_REQUEST,
        }
    } else {
        event!(Level::ERROR, rejection = ?err, "Unhandled rejection");
        Error {
            error: ErrorBody::Internal,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&error),
        error.status,
    ))
}
