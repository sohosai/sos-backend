use std::fmt::{self, Debug};
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures::stream::{Stream, StreamExt, TryStreamExt};

pub struct ByteStream(Pin<Box<dyn Stream<Item = anyhow::Result<Bytes>> + Send + 'static>>);

impl Debug for ByteStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("ByteStream").finish()
    }
}

impl Stream for ByteStream {
    type Item = anyhow::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx)
    }
}

impl ByteStream {
    pub fn new<S, E>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, E>> + Send + 'static,
        E: Into<anyhow::Error> + Send + 'static,
    {
        ByteStream(Box::pin(stream.map_err(Into::into)))
    }
}
