use std::convert::TryInto;

use std::fmt::{self, Debug};
use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::Result;
use bytes::Bytes;
use futures::stream::{Stream, StreamExt};

struct ObjectDataStream {
    size: Option<u64>,
    stream: Pin<Box<dyn Stream<Item = Result<Bytes>> + Send + Sync + 'static>>,
}

impl Debug for ObjectDataStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ObjectData")
            .field("data", &"..")
            .field("size", &self.size)
            .finish()
    }
}

impl Stream for ObjectDataStream {
    type Item = Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.size.and_then(|size| size.try_into().ok()))
    }
}

#[derive(Debug)]
pub struct ObjectData(ObjectDataStream);

impl ObjectData {
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes>> + Send + Sync + 'static,
    {
        ObjectData(ObjectDataStream {
            stream: Box::pin(stream),
            size: None,
        })
    }

    pub fn from_stream_with_size<S>(stream: S, size: u64) -> Self
    where
        S: Stream<Item = Result<Bytes>> + Send + Sync + 'static,
    {
        ObjectData(ObjectDataStream {
            stream: Box::pin(stream),
            size: Some(size),
        })
    }

    pub fn size(&self) -> Option<u64> {
        self.0.size
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<Bytes>> + Send + Sync + 'static {
        self.0
    }
}
