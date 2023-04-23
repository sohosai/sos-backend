use std::convert::TryInto;
use std::fmt::{self, Debug};

use anyhow::Context;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{
    future,
    stream::{self, Stream, TryStreamExt},
};
use rusoto_core::RusotoError;
use rusoto_s3::{S3Client, S3};
use sos21_domain::context::ObjectRepository;
use sos21_domain::model::object::{Object, ObjectData, ObjectId};
use thiserror::Error;

// TODO: Tune buffer size and initial capacity
const MINIMUM_PART_SIZE: usize = 10 * 1024 * 1024;
const INITIAL_BUFFER_SIZE: usize = 11 * 1024 * 1024;

pub struct ObjectS3 {
    pub bucket: String,
    pub client: S3Client,
}

impl Debug for ObjectS3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // `S3Client` does't implement `Debug`,
        // so using an unit struct below as a replacement.
        #[derive(Debug)]
        struct S3Client;

        f.debug_struct("ObjectS3")
            .field("bucket", &self.bucket)
            .field("client", &S3Client)
            .finish()
    }
}

#[derive(Debug, Error, Clone)]
#[error("out of limit object size")]
pub struct OutOfLimitSizeError {
    _priv: (),
}

// We don't have transaction and consistency check between S3 and database, and
// we are currently maintaining consistency and overlooking the lack of
// atomicity and isolation by not making any deletions.
// Be warned if you attempt to implement deletion.
#[async_trait::async_trait]
impl ObjectRepository for ObjectS3 {
    type OutOfLimitSizeError = OutOfLimitSizeError;

    async fn store_object(&self, object: Object) -> anyhow::Result<()> {
        self.store_object_impl(object, None).await?;
        Ok(())
    }

    async fn store_object_with_limit(
        &self,
        object: Object,
        limit: u64,
    ) -> anyhow::Result<Result<(), OutOfLimitSizeError>> {
        if let StoreObjectResult::OutOfLimit = self.store_object_impl(object, Some(limit)).await? {
            Ok(Err(OutOfLimitSizeError { _priv: () }))
        } else {
            Ok(Ok(()))
        }
    }

    async fn get_object(&self, id: ObjectId) -> anyhow::Result<Option<Object>> {
        let object_key = to_object_key(id);

        let request = rusoto_s3::GetObjectRequest {
            bucket: self.bucket.clone(),
            key: object_key,
            ..Default::default()
        };
        let result = self.client.get_object(request).await;
        if let Err(RusotoError::Service(rusoto_s3::GetObjectError::NoSuchKey(_))) = result {
            return Ok(None);
        }

        let body = result?
            .body
            .context("No body in the response of GetObject")?
            .map_err(anyhow::Error::new);

        Ok(Some(Object {
            id,
            data: ObjectData::from_stream(body),
        }))
    }
}

enum StoreObjectResult {
    OutOfLimit,
    Stored,
}

impl ObjectS3 {
    async fn store_object_impl(
        &self,
        object: Object,
        size_limit: Option<u64>,
    ) -> anyhow::Result<StoreObjectResult> {
        let object_key = to_object_key(object.id);

        let create_multipart_request = rusoto_s3::CreateMultipartUploadRequest {
            bucket: self.bucket.clone(),
            key: object_key.clone(),
            ..Default::default()
        };
        let create_multipart_output = self
            .client
            .create_multipart_upload(create_multipart_request)
            .await?;

        let upload_id = create_multipart_output
            .upload_id
            .context("No upload_id in the response of CreateMultipartUpload")?;

        let input = UploadInput {
            client: &self.client,
            bucket: self.bucket.clone(),
            key: object_key.clone(),
            upload_id: upload_id.clone(),
            size_limit,
            data: object.data.into_stream(),
        };

        let upload_result = upload(input).await;
        match upload_result {
            Err(_) | Ok(StoreObjectResult::OutOfLimit) => {
                let abort_multipart_request = rusoto_s3::AbortMultipartUploadRequest {
                    bucket: self.bucket.clone(),
                    key: object_key.clone(),
                    upload_id,
                    ..Default::default()
                };
                self.client
                    .abort_multipart_upload(abort_multipart_request)
                    .await
                    .context(if let Err(err) = &upload_result {
                        format!("Failed to abort multipart upload (abort cause: {})", err)
                    } else {
                        "Failed to abort multipart upload on exceeding the limit".to_owned()
                    })?;
            }
            _ => {}
        }

        upload_result
    }
}

struct UploadInput<'a, S> {
    client: &'a S3Client,
    bucket: String,
    key: String,
    upload_id: String,
    size_limit: Option<u64>,
    data: S,
}

async fn upload<S>(input: UploadInput<'_, S>) -> anyhow::Result<StoreObjectResult>
where
    S: Stream<Item = anyhow::Result<Bytes>> + Send + Unpin + 'static,
{
    let UploadInput {
        client,
        bucket,
        key,
        upload_id,
        mut data,
        size_limit,
    } = input;

    let mut buf = BytesMut::with_capacity(INITIAL_BUFFER_SIZE);
    let mut part_number = 0;
    let mut part_futures = Vec::new();
    let mut total_size: u64 = 0;

    while let Some(chunk) = data.try_next().await? {
        let chunk_len: u64 = chunk.len().try_into()?;
        total_size += chunk_len;

        if let Some(size_limit) = size_limit {
            if total_size > size_limit {
                return Ok(StoreObjectResult::OutOfLimit);
            }
        }

        buf.put(chunk);
        if buf.len() >= MINIMUM_PART_SIZE {
            let body = buf.copy_to_bytes(buf.len());
            let input = UploadPartInput {
                client: client.clone(),
                bucket: bucket.clone(),
                key: key.clone(),
                upload_id: upload_id.clone(),
                part_number,
                body,
            };
            part_futures.push(tokio::spawn(upload_part(input)));
            part_number += 1;
        }
    }

    if buf.has_remaining() {
        let body = buf.freeze();
        let input = UploadPartInput {
            client: client.clone(),
            bucket: bucket.clone(),
            key: key.clone(),
            upload_id: upload_id.clone(),
            part_number,
            body,
        };
        part_futures.push(tokio::spawn(upload_part(input)));
    }

    let completed_parts = future::try_join_all(part_futures)
        .await?
        .into_iter()
        .collect::<anyhow::Result<_>>()?;

    let complete_request = rusoto_s3::CompleteMultipartUploadRequest {
        bucket,
        key,
        upload_id,
        multipart_upload: Some(rusoto_s3::CompletedMultipartUpload {
            parts: Some(completed_parts),
        }),
        ..Default::default()
    };
    client.complete_multipart_upload(complete_request).await?;

    Ok(StoreObjectResult::Stored)
}

struct UploadPartInput {
    client: S3Client,
    bucket: String,
    key: String,
    upload_id: String,
    part_number: i64,
    body: Bytes,
}

async fn upload_part(input: UploadPartInput) -> anyhow::Result<rusoto_s3::CompletedPart> {
    let UploadPartInput {
        client,
        bucket,
        key,
        upload_id,
        part_number,
        body,
    } = input;

    let content_length: i64 = body.len().try_into()?;
    let body = rusoto_core::ByteStream::new(stream::once(async move { Ok(body) }));
    let upload_request = rusoto_s3::UploadPartRequest {
        bucket,
        key,
        body: Some(body),
        content_length: Some(content_length),
        part_number,
        upload_id,
        ..Default::default()
    };
    let output = client.upload_part(upload_request).await?;
    let e_tag = output
        .e_tag
        .context("No e_tag in the response of UploadPart")?;

    Ok(rusoto_s3::CompletedPart {
        e_tag: Some(e_tag),
        part_number: Some(input.part_number),
    })
}

fn to_object_key(id: ObjectId) -> String {
    id.to_uuid().to_hyphenated().to_string()
}
