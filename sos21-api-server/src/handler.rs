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
    InvalidEmailAddress,
    NotUniversityEmailAddress,
    ServiceUnavailable(anyhow::Error),
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
            HandlerError::InvalidEmailAddress => ErasedHandlerError::InvalidEmailAddress,
            HandlerError::NotUniversityEmailAddress => {
                ErasedHandlerError::NotUniversityEmailAddress
            }
            HandlerError::ServiceUnavailable(err) => ErasedHandlerError::ServiceUnavailable(err),
            HandlerError::Server(err) => ErasedHandlerError::Server(err),
        }
    }
}

impl warp::reject::Reject for ErasedHandlerError {}

fn handle_raw_handler_result<R, E>(result: HandlerResult<R, E>) -> Result<R, Rejection>
where
    R: Reply,
    E: HandlerResponse,
{
    result.map_err(|error| warp::reject::custom(crate::handler::ErasedHandlerError::from(error)))
}

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

macro_rules! raw_response_handler {
    ($vis:vis async fn $name:ident (
        $ctx:ident: Authentication<Context>
        $(, $param:ident : $ty:ty)* $(,)?
    ) -> HandlerResult<impl warp::Reply, $err:ty> $body:block) => {
        handler! {
            @impl_authentication $vis $name($ctx, $($param: $ty),*) -> impl warp::Reply, $err, $body;
              crate::handler::handle_raw_handler_result
        }
    };
    ($vis:vis async fn $name:ident (
        $ctx:ident: Login<Context>
        $(, $param:ident : $ty:ty)* $(,)?
    ) -> HandlerResult<impl warp::Reply, $err:ty> $body:block) => {
        handler! {
            @impl_login $vis $name($ctx, $($param: $ty),*) -> impl warp::Reply, $err, $body;
              crate::handler::handle_raw_handler_result
        }
    };
    ($vis:vis async fn $name:ident (
        $ctx:ident: Context
        $(, $param:ident : $ty:ty)* $(,)?
    ) -> HandlerResult<impl warp::Reply, $err:ty> $body:block) => {
        $vis async fn $name(
            app: crate::app::App
            $(, $param: $ty)*
        ) -> Result<impl ::warp::reply::Reply, ::warp::reject::Rejection> {
            async fn run(
                app: crate::app::App
                $(, $param: $ty)*
            ) -> HandlerResult<impl warp::Reply, $err> {
                let $ctx: Context = app.start_context().await?;
                let result: HandlerResult<_, $err> = $body;
                $ctx.commit_changes().await?;
                result
            }

            crate::handler::handle_raw_handler_result(run(app $(, $param)*).await)
        }
    };
}

macro_rules! handler {
    ($vis:vis async fn $name:ident (
        $ctx:ident: Authentication<Context>
        $(, $param:ident : $ty:ty)* $(,)?
    ) -> HandlerResult<$resp:ty, $err:ty> $body:block) => {
        handler! {
            @impl_authentication $vis $name($ctx, $($param: $ty),*) -> $resp, $err, $body;
              crate::handler::handle_handler_result
        }
    };
    ($vis:vis async fn $name:ident (
        $ctx:ident: Login<Context>
        $(, $param:ident : $ty:ty)* $(,)?
    ) -> HandlerResult<$resp:ty, $err:ty> $body:block) => {
        handler! {
                @impl_login $vis $name($ctx, $($param: $ty),*) -> $resp, $err, $body;
              crate::handler::handle_handler_result
        }
    };
    ($vis:vis async fn $name:ident (
        $ctx:ident: Context
        $(, $param:ident : $ty:ty)* $(,)?
    ) -> HandlerResult<$resp:ty, $err:ty> $body:block) => {
        $vis async fn $name(
            app: crate::app::App
            $(, $param: $ty)*
        ) -> Result<impl ::warp::reply::Reply, ::warp::reject::Rejection> {
            async fn run(
                app: crate::app::App
                $(, $param: $ty)*
            ) -> HandlerResult<$resp, $err> {
                let $ctx: Context = app.start_context().await?;
                let result: HandlerResult<_, $err> = $body;
                $ctx.commit_changes().await?;
                result
            }

            crate::handler::handle_handler_result(run(app $(, $param)*).await)
        }
    };
    ($vis:vis async fn $name:ident (
        $app:ident: App
        $(, $param:ident : $ty:ty)* $(,)?
    ) -> HandlerResult<$resp:ty, $err:ty> $body:block) => {
        $vis async fn $name(
            app: App
            $(, $param: $ty)*
        ) -> Result<impl ::warp::reply::Reply, ::warp::reject::Rejection> {
            async fn run(
                $app: App
                $(, $param: $ty)*
            ) -> HandlerResult<$resp, $err> {
                $body
            }
            crate::handler::handle_handler_result(run(app $(, $param)*).await)
        }
    };
    ($vis:vis async fn $name:ident (
        $($param:ident : $ty:ty),* $(,)?
    ) -> HandlerResult<$resp:ty, $err:ty> $body:block) => {
        $vis async fn $name(
            $($param: $ty),*
        ) -> Result<impl ::warp::reply::Reply, ::warp::reject::Rejection> {
            let result: HandlerResult<$resp, $err> = $body;
            crate::handler::handle_handler_result(result)
        }
    };
    (@impl_authentication $vis:vis $name:ident (
            $ctx:ident,
            $($param:ident : $ty:ty),*
     ) -> $resp:ty, $err:ty, $body:block; $handle:path
    ) => {
        handler! {
            @impl $vis $name (
                (auth, ctx) $ctx = Authentication::<Context>::new(ctx, auth.user_id, auth.email)?,
                $($param: $ty),*
            ) -> $resp, $err, $body; $handle
        }
    };
    (@impl_login $vis:vis $name:ident (
            $ctx:ident,
            $($param:ident : $ty:ty),*
     ) -> $resp:ty, $err:ty, $body:block; $handle:path
    ) => {
        handler! {
            @impl $vis $name (
                (auth, ctx) $ctx = {
                    let ctx = ::sos21_domain::context::Authentication::new(
                        ctx,
                        auth.user_id,
                        auth.email
                    )?;
                    Login::<Context>::new(ctx).await?
                },
                $($param: $ty),*
            ) -> $resp, $err, $body; $handle
        }
    };
    (@impl $vis:vis $name:ident (
            ($auth_bind:ident, $ctx_bind:ident) $ctx:ident = $make_ctx:expr,
            $($param:ident : $ty:ty),*
     ) -> $resp:ty, $err:ty, $body:block; $handle:path
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
                let result: HandlerResult<_, $err> = $body;
                $ctx.into_inner().commit_changes().await?;
                result
            }

            $handle(run(app, auth $(, $param)*).await)
        }
    };
}

pub mod file;
pub mod file_distribution;
pub mod file_sharing;
pub mod form;
pub mod form_answer;
pub mod me;
pub mod meta;
pub mod pending_project;
pub mod project;
pub mod registration_form;
pub mod registration_form_answer;
pub mod user;
pub mod user_invitation;

pub mod signup;
pub use signup::handler as signup;

pub mod invite_user;
pub use invite_user::handler as invite_user;

pub mod assign_user_role_to_email;
pub use assign_user_role_to_email::handler as assign_user_role_to_email;

pub mod project_creation_availability;

pub trait HandlerResponse: Serialize {
    /// Server errors are returned as `anyhow::Error`, not as `HandlerResponse`.
    /// Thus, it always stands that `!x.status_code().is_server_error()`.
    fn status_code(&self) -> StatusCode;
}

#[derive(Debug)]
pub enum HandlerError<E> {
    Client(E),
    NotSignedUp,
    InvalidEmailAddress,
    NotUniversityEmailAddress,
    ServiceUnavailable(anyhow::Error),
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

impl<E> From<sos21_domain::context::authentication::AuthenticationError> for HandlerError<E> {
    fn from(e: sos21_domain::context::authentication::AuthenticationError) -> HandlerError<E> {
        use sos21_domain::context::authentication::AuthenticationError;
        match e {
            AuthenticationError::InvalidEmailAddress => HandlerError::InvalidEmailAddress,
            AuthenticationError::NotUniversityEmailAddress => {
                HandlerError::NotUniversityEmailAddress
            }
        }
    }
}

impl<E> From<sos21_domain::context::login::LoginError> for HandlerError<E> {
    fn from(e: sos21_domain::context::login::LoginError) -> HandlerError<E> {
        use sos21_domain::context::login::LoginError;
        match e {
            LoginError::NotSignedUp => HandlerError::NotSignedUp,
            LoginError::Internal(e) => HandlerError::Server(e),
        }
    }
}
