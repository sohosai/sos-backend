use std::fmt::{self, Debug};

use rusoto_s3::S3Client;

mod object_repository;
use object_repository::ObjectS3;

#[derive(Clone)]
pub struct S3 {
    object_bucket: String,
    client: S3Client,
}

impl Debug for S3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // `S3Client` does't implement `Debug`,
        // so using an unit struct below as a replacement.
        #[derive(Debug)]
        struct S3Client;

        f.debug_struct("S3")
            .field("object_bucket", &self.object_bucket)
            .field("client", &S3Client)
            .finish()
    }
}

impl S3 {
    pub fn new(client: S3Client, object_bucket: impl Into<String>) -> Self {
        S3 {
            object_bucket: object_bucket.into(),
            client,
        }
    }
}

sos21_domain::delegate_object_repository! {
    impl ObjectRepository for S3 {
        Self { ObjectS3 },
        // TODO: Reduce clone() which is too much for the temporary
        self { ObjectS3 { bucket: self.object_bucket.clone(), client: self.client.clone() } }
    }
}
