use crate::handler::model::user::UserId;

use chrono::{DateTime, Utc};
use mime::Mime;
use serde::{Deserialize, Serialize};
use sos21_use_case::model::file as use_case;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileId(pub Uuid);

impl FileId {
    pub fn from_use_case(id: use_case::FileId) -> FileId {
        FileId(id.0)
    }

    pub fn into_use_case(self) -> use_case::FileId {
        use_case::FileId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: FileId,
    pub created_at: DateTime<Utc>,
    pub author_id: UserId,
    pub name: Option<String>,
    #[serde(with = "serde_mime", rename = "type")]
    pub type_: Mime,
    #[serde(with = "hex::serde")]
    pub blake3_digest: [u8; 32],
    pub size: u64,
}

impl File {
    pub fn from_use_case(file: use_case::File) -> Self {
        File {
            id: FileId::from_use_case(file.id),
            created_at: file.created_at,
            author_id: UserId::from_use_case(file.author_id),
            name: file.name,
            type_: file.type_,
            blake3_digest: file.blake3_digest,
            size: file.size,
        }
    }
}

mod serde_mime {
    use mime::Mime;
    use serde::{
        de::{self, Deserialize, Deserializer},
        ser::Serializer,
    };

    pub fn serialize<S>(mime: &Mime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(mime)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Mime, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
