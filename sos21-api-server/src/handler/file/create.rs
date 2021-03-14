use std::sync::Arc;
use std::task::Poll;

use crate::app::Context;
use crate::handler::model::file::File;
use crate::handler::{HandlerError, HandlerResponse, HandlerResult};

use anyhow::Context as _;
use bytes::Buf;
use futures::{
    future::{self, TryFutureExt},
    stream::{self, Stream, StreamExt, TryStreamExt},
};
use mime::Mime;
use mpart_async::server::MultipartStream;
use serde::Serialize;
use sos21_domain::context::Login;
use sos21_use_case::create_file;
use tokio::sync::Notify;
use warp::http::StatusCode;

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub files: Vec<ResponseFile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseFile {
    pub name: String,
    pub file: File,
}

impl HandlerResponse for Response {
    fn status_code(&self) -> StatusCode {
        StatusCode::CREATED
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "type")]
pub enum Error {
    InvalidFileName,
    OutOfFileUsageQuota,
    InsufficientPermissions,
    NoBoundaryInContentType,
    NoNameDirectiveInPart,
    InvalidContentTypeInPart,
}

impl HandlerResponse for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidFileName => StatusCode::BAD_REQUEST,
            Error::OutOfFileUsageQuota => StatusCode::CONFLICT,
            Error::InsufficientPermissions => StatusCode::FORBIDDEN,
            Error::NoBoundaryInContentType => StatusCode::BAD_REQUEST,
            Error::NoNameDirectiveInPart => StatusCode::BAD_REQUEST,
            Error::InvalidContentTypeInPart => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<create_file::Error> for Error {
    fn from(err: create_file::Error) -> Error {
        match err {
            create_file::Error::InvalidName => Error::InvalidFileName,
            create_file::Error::OutOfUsageQuota => Error::OutOfFileUsageQuota,
            create_file::Error::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[apply_macro::apply(handler)]
pub async fn handler(
    ctx: Login<Context>,
    mime: Mime,
    body: impl Stream<Item = Result<impl Buf, warp::Error>> + Send + Sync + Unpin + 'static,
) -> HandlerResult<Response, Error> {
    let boundary = mime
        .get_param("boundary")
        .ok_or(HandlerError::Client(Error::NoBoundaryInContentType))?
        .to_string();

    let mut data = MultipartStream::new(
        boundary,
        body.map_ok(|mut buf| buf.copy_to_bytes(buf.remaining())),
    );

    let mut files = Vec::new();
    let mut file_futures = Vec::new();

    while let Some(part) = data.try_next().await.context("Failed reading form data")? {
        let name = part
            .name()
            .map_err(|_| HandlerError::Client(Error::NoNameDirectiveInPart))?
            .to_owned();
        let content_type = part
            .content_type()
            .ok()
            .map(|type_| type_.parse())
            .transpose()
            .map_err(|_| HandlerError::Client(Error::InvalidContentTypeInPart))?;
        let filename = part.filename().ok().map(|filename| filename.to_owned());

        // We need to wait `create_file` reading `part` to the end before reading the next part
        let read_finished = Arc::new(Notify::new());
        let end_notifier = {
            let read_notifier = Arc::clone(&read_finished);
            stream::poll_fn(move |_| {
                read_notifier.notify_one();
                Poll::Ready(None)
            })
        };
        let input = create_file::Input {
            data: part.map_err(anyhow::Error::msg).chain(end_notifier),
            name: filename,
            content_type,
        };

        // TODO: Gain more concurrency by awaiting on `file_futures` here
        let read_fut = read_finished.notified();
        futures::pin_mut!(read_fut);
        match future::select(read_fut, Box::pin(create_file::run(&ctx, input))).await {
            future::Either::Left(((), create_fut)) => {
                file_futures.push(create_fut.map_ok(|file| {
                    let file = File::from_use_case(file);
                    ResponseFile { name, file }
                }));
            }
            future::Either::Right((file, read_fut)) => {
                let file = File::from_use_case(file?);
                files.push(ResponseFile { name, file });
                read_fut.await;
            }
        }
    }

    files.extend(future::try_join_all(file_futures).await?);

    Ok(Response { files })
}
