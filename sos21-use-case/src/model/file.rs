use crate::model::user::UserId;

use chrono::{DateTime, Utc};
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
            size: file.size,
        }
    }
}
