use serde::Serialize;
use sos21_use_case::UseCaseError;
use warp::http::StatusCode;

pub mod model;

pub mod health;
pub mod project;
pub mod signup;
pub mod user;
pub use signup::handler as signup;
pub mod me;
pub use me::handler as me;

pub trait HandlerResponse: Serialize {
    /// Server errors are returned as `anyhow::Error`, not as `HandlerResponse`.
    /// Thus, it always stands that `!x.status_code().is_server_error()`.
    fn status_code(&self) -> StatusCode;
}

#[derive(Debug)]
pub enum HandlerError<E> {
    Client(E),
    Server(anyhow::Error),
}

pub type HandlerResult<T, E> = Result<T, HandlerError<E>>;

impl<E1, E2> From<UseCaseError<E1>> for HandlerError<E2>
where
    E1: Into<E2>,
{
    fn from(e: UseCaseError<E1>) -> HandlerError<E2> {
        match e {
            UseCaseError::UseCase(e) => HandlerError::Client(e.into()),
            UseCaseError::Internal(e) => HandlerError::Server(e),
        }
    }
}
