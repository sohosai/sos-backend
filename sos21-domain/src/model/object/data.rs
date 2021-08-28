use std::convert::TryInto;
use std::fmt::{self, Debug};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::{Context as _, Result};
use bytes::Bytes;
use futures::{
    channel::oneshot,
    future::FutureExt,
    stream::{Stream, StreamExt},
};

/// Summary of [`ObjectData`].
#[derive(Debug, Clone)]
pub struct ObjectDataSummary {
    pub number_of_bytes: u64,
    pub blake3_digest: [u8; 32],
}

struct ObjectDataStream {
    summary_sender: Option<oneshot::Sender<ObjectDataSummary>>,
    acc_size: u64,
    acc_hasher: blake3::Hasher,
    stream: Pin<Box<dyn Stream<Item = Result<Bytes>> + Send + 'static>>,
}

impl ObjectDataStream {
    fn new<S>(stream: S, summary_sender: Option<oneshot::Sender<ObjectDataSummary>>) -> Self
    where
        S: Stream<Item = Result<Bytes>> + Send + 'static,
    {
        ObjectDataStream {
            summary_sender,
            acc_size: 0,
            acc_hasher: blake3::Hasher::new(),
            stream: Box::pin(stream),
        }
    }
}

impl Debug for ObjectDataStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ObjectDataStream")
            .field("acc_size", &self.acc_size)
            .field("acc_hasher", &self.acc_hasher)
            .field("summary_sender", &self.summary_sender)
            .finish()
    }
}

impl Stream for ObjectDataStream {
    type Item = Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match futures::ready!(self.stream.poll_next_unpin(cx)) {
            Some(Ok(bytes)) => {
                if self.summary_sender.is_some() {
                    let len: u64 = bytes.len().try_into()?;
                    self.acc_size += len;
                    // TODO: Consider running on a thread where blocking is acceptable
                    self.acc_hasher.update(&bytes);
                }
                Poll::Ready(Some(Ok(bytes)))
            }
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => {
                if let Some(sender) = self.summary_sender.take() {
                    let summary = ObjectDataSummary {
                        number_of_bytes: self.acc_size,
                        blake3_digest: self.acc_hasher.finalize().into(),
                    };
                    let _ = sender.send(summary);
                }
                Poll::Ready(None)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

#[derive(Debug)]
pub struct ObjectData(ObjectDataStream);

/// Future that recieves [`ObjectDataSummary`] after reading all the data in [`ObjectData`].
#[derive(Debug)]
pub struct ObjectDataSummaryReceiver(oneshot::Receiver<ObjectDataSummary>);

impl Future for ObjectDataSummaryReceiver {
    type Output = Result<ObjectDataSummary>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.0.poll_unpin(cx).map(|result| {
            result.context("ObjectDataSummaryReceiver polled after the stream is failed")
        })
    }
}

impl ObjectData {
    /// Creates `ObjectData` by wrapping a stream.
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes>> + Send + 'static,
    {
        ObjectData(ObjectDataStream::new(stream, None))
    }

    /// Creates `ObjectData` by wrapping a stream and returns a `Future` that recieves
    /// [`ObjectDataSummary`] after reading all the object data.
    ///
    /// We don't want to have all of the `ObjectData` in memory at once, so we compute the summary
    /// of the content (currently size and digest) as we read the data, and return it when the
    /// reading is complete.
    pub fn from_stream_with_summary<S>(stream: S) -> (Self, ObjectDataSummaryReceiver)
    where
        S: Stream<Item = Result<Bytes>> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let receiver = ObjectDataSummaryReceiver(rx);
        (
            ObjectData(ObjectDataStream::new(stream, Some(tx))),
            receiver,
        )
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<Bytes>> + Send + 'static {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::test::model as test_model;
    use futures::stream::TryStreamExt;

    #[tokio::test]
    async fn test_summary_size_sum() {
        let (data, _digest, _size, receiver) = test_model::new_object_data_with_summary();
        let mut total_size = 0;
        let mut stream = data.into_stream();
        while let Some(chunk) = stream.try_next().await.unwrap() {
            total_size += chunk.len();
        }
        let summary = receiver.await.unwrap();
        assert_eq!(summary.number_of_bytes, total_size as u64);
    }

    #[tokio::test]
    async fn test_summary_size_expected() {
        let (data, _digest, expected_size, receiver) = test_model::new_object_data_with_summary();
        // consume the stream
        data.into_stream().try_collect::<Vec<_>>().await.unwrap();
        let summary = receiver.await.unwrap();
        assert_eq!(summary.number_of_bytes, expected_size);
    }

    #[tokio::test]
    async fn test_summary_digest_sum() {
        let (data, _digest, _size, receiver) = test_model::new_object_data_with_summary();
        let mut hasher = blake3::Hasher::new();
        let mut stream = data.into_stream();
        while let Some(chunk) = stream.try_next().await.unwrap() {
            hasher.update(&chunk);
        }
        let digest: [u8; 32] = hasher.finalize().into();
        let summary = receiver.await.unwrap();
        assert_eq!(summary.blake3_digest, digest);
    }

    #[tokio::test]
    async fn test_summary_digest_expected() {
        let (data, expected_digest, _size, receiver) = test_model::new_object_data_with_summary();
        // consume the stream
        data.into_stream().try_collect::<Vec<_>>().await.unwrap();
        let summary = receiver.await.unwrap();
        assert_eq!(summary.blake3_digest, expected_digest);
    }
}
