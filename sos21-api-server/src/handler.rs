use serde::Serialize;
use sos21_use_case::UseCaseError;
use warp::{http::StatusCode, reject::Rejection, reply::Reply};

pub mod model;

/// Type erased [`HandlerError`].
#[derive(Debug)]
pub enum ErasedHandlerError {
    Client {
        status_code: StatusCode,
        info: serde_json::Value,
    },
    NotSignedUp,
    InvalidEmail,
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
            HandlerError::NotSignedUp => ErasedHandlerError::NotSignedUp,
            HandlerError::InvalidEmail => ErasedHandlerError::InvalidEmail,
            HandlerError::Server(err) => ErasedHandlerError::Server(err),
        }
    }
}

impl warp::reject::Reject for ErasedHandlerError {}

fn handle_handler_result<R, E>(result: HandlerResult<R, E>) -> Result<impl Reply, Rejection>
where
    R: HandlerResponse,
    E: HandlerResponse,
{
    match result {
        Ok(response) => {
            let code = response.status_code();
            debug_assert!(code.is_success());
            Ok(warp::reply::with_status(warp::reply::json(&response), code))
        }
        Err(error) => Err(warp::reject::custom(
            crate::handler::ErasedHandlerError::from(error),
        )),
    }
}

macro_rules! handler {
    ($vis:vis async fn $name:ident (
        $ctx:ident: Authentication<Context>
        $(, $param:ident : $ty:ty)* $(,)?
    ) -> HandlerResult<$resp:ty, $err:ty> $body:block) => {
        handler! {
            @impl $vis $name (
                (auth, ctx) $ctx = Authentication::<Context>::new(ctx, auth.user_id, auth.email)?,
                $($param: $ty),*
            ) -> $resp, $err, $body
        }
    };
    ($vis:vis async fn $name:ident (
        $ctx:ident: Login<Context>
        $(, $param:ident : $ty:ty)* $(,)?
    ) -> HandlerResult<$resp:ty, $err:ty> $body:block) => {
        handler! {
            @impl $vis $name (
                (auth, ctx) $ctx = {
                    let ctx = ::sos21_domain_context::Authentication::new(
                        ctx,
                        auth.user_id,
                        auth.email
                    )?;
                    Login::<Context>::new(ctx).await?
                },
                $($param: $ty),*
            ) -> $resp, $err, $body
        }
    };
    ($vis:vis async fn $name:ident (
        $($param:ident : $ty:ty),* $(,)?
    ) -> HandlerResult<$resp:ty, $err:ty> $body:block) => {
        $vis async fn $name(
            $(, $param: $ty)*
        ) -> Result<impl ::warp::reply::Reply, ::warp::reject::Rejection> {
            let result: HandlerResult<$resp, $err> = $body;
            crate::handler::handle_handler_result(result)
        }
    };
    (@impl $vis:vis $name:ident (
            ($auth_bind:ident, $ctx_bind:ident) $ctx:ident = $make_ctx:expr,
            $($param:ident : $ty:ty),*
     ) -> $resp:ty, $err:ty, $body:block
    ) => {
        $vis async fn $name(
            app: crate::app::App,
            auth: crate::filter::AuthenticationInfo
            $(, $param: $ty)*
        ) -> Result<impl ::warp::reply::Reply, ::warp::reject::Rejection> {
            async fn run(
                app: crate::app::App,
                $auth_bind: crate::filter::AuthenticationInfo
                $(, $param: $ty)*
            ) -> HandlerResult<$resp, $err> {
                let $ctx_bind = app.start_context().await?;
                let $ctx = $make_ctx;
                let result: HandlerResult<$resp, $err> = $body;
                $ctx.into_inner().commit_changes().await?;
                result
            }

            crate::handler::handle_handler_result(run(app, auth $(, $param)*).await)
        }
    };
}

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
    NotSignedUp,
    InvalidEmail,
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

impl<E> From<anyhow::Error> for HandlerError<E> {
    fn from(e: anyhow::Error) -> HandlerError<E> {
        HandlerError::Server(e)
    }
}

impl<E> From<sos21_domain_context::authentication::AuthenticationError> for HandlerError<E> {
    fn from(e: sos21_domain_context::authentication::AuthenticationError) -> HandlerError<E> {
        use sos21_domain_context::authentication::AuthenticationError;
        match e {
            AuthenticationError::InvalidEmail => HandlerError::InvalidEmail,
        }
    }
}

impl<E> From<sos21_domain_context::login::LoginError> for HandlerError<E> {
    fn from(e: sos21_domain_context::login::LoginError) -> HandlerError<E> {
        use sos21_domain_context::login::LoginError;
        match e {
            LoginError::NotSignedUp => HandlerError::NotSignedUp,
            LoginError::Internal(e) => HandlerError::Server(e),
        }
    }
}
