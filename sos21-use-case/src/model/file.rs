use std::fmt::{self, Debug};
use std::pin::Pin;

use crate::model::user::UserId;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::stream::Stream;
use mime::Mime;
use sos21_domain::model::file as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub Uuid);

impl FileId {
    pub fn from_entity(id: entity::FileId) -> FileId {
        FileId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::FileId {
        entity::FileId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub id: FileId,
    pub created_at: DateTime<Utc>,
    pub author_id: UserId,
    pub name: Option<String>,
    pub type_: Mime,
    pub blake3_digest: [u8; 32],
    pub size: u64,
}

impl File {
    pub fn from_entity(file: entity::File) -> Self {
        File {
            id: FileId::from_entity(file.id),
            created_at: file.created_at.utc(),
            author_id: UserId::from_entity(file.author_id),
            name: file.name.map(entity::FileName::into_string),
            type_: file.type_.into_mime(),
            blake3_digest: file.blake3_digest.into_array(),
            size: file.size.to_number_of_bytes(),
        }
    }
}

pub struct FileObject {
    pub file: File,
    pub object_data: Pin<Box<dyn Stream<Item = anyhow::Result<Bytes>> + Send + Sync + 'static>>,
}

impl Debug for FileObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FileObject")
            .field("file", &self.file)
            .field("object_data", &"..")
            .finish()
    }
}

impl FileObject {
    pub fn from_entity(file: entity::File, object: sos21_domain::model::object::Object) -> Self {
        FileObject {
            file: File::from_entity(file),
            object_data: Box::pin(object.data.into_stream()),
        }
    }
}
