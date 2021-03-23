use crate::model::file::FileId;
use crate::model::form::FormId;
use crate::model::project::ProjectId;

use chrono::{DateTime, Utc};
use mime::Mime;
use sos21_domain::model::file_sharing as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileSharingId(pub Uuid);

impl FileSharingId {
    pub fn from_entity(id: entity::FileSharingId) -> Self {
        FileSharingId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::FileSharingId {
        entity::FileSharingId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSharingScope {
    Project(ProjectId),
    FormAnswer(ProjectId, FormId),
    Committee,
    CommitteeOperator,
    Public,
}

impl FileSharingScope {
    pub fn from_entity(scope: entity::FileSharingScope) -> Self {
        match scope {
            entity::FileSharingScope::Project(project_id) => {
                FileSharingScope::Project(ProjectId::from_entity(project_id))
            }
            entity::FileSharingScope::FormAnswer(project_id, form_id) => {
                FileSharingScope::FormAnswer(
                    ProjectId::from_entity(project_id),
                    FormId::from_entity(form_id),
                )
            }
            entity::FileSharingScope::Committee => FileSharingScope::Committee,
            entity::FileSharingScope::CommitteeOperator => FileSharingScope::CommitteeOperator,
            entity::FileSharingScope::Public => FileSharingScope::Public,
        }
    }

    pub fn into_entity(self) -> entity::FileSharingScope {
        match self {
            FileSharingScope::Project(project_id) => {
                entity::FileSharingScope::Project(project_id.into_entity())
            }
            FileSharingScope::FormAnswer(project_id, form_id) => {
                entity::FileSharingScope::FormAnswer(
                    project_id.into_entity(),
                    form_id.into_entity(),
                )
            }
            FileSharingScope::Committee => entity::FileSharingScope::Committee,
            FileSharingScope::CommitteeOperator => entity::FileSharingScope::CommitteeOperator,
            FileSharingScope::Public => entity::FileSharingScope::Public,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSharing {
    pub id: FileSharingId,
    pub created_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: FileSharingScope,
    pub file_id: FileId,
    pub file_name: Option<String>,
    pub file_type: Mime,
    pub file_size: u64,
}

impl FileSharing {
    pub fn from_entity(
        sharing: entity::FileSharing,
        file: sos21_domain::model::file::File,
    ) -> Self {
        let sharing = sharing.into_content();
        FileSharing {
            id: FileSharingId::from_entity(sharing.id),
            created_at: sharing.created_at.utc(),
            is_revoked: sharing.is_revoked,
            expires_at: sharing.expires_at.map(|expires_at| expires_at.utc()),
            scope: FileSharingScope::from_entity(sharing.scope),
            file_id: FileId::from_entity(sharing.file_id),
            file_name: file.name.map(|name| name.into_string()),
            file_type: file.type_.into_mime(),
            file_size: file.size.to_number_of_bytes(),
        }
    }
}
