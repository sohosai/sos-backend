use crate::handler::model::date_time::DateTime;
use crate::handler::model::file::FileId;
use crate::handler::model::form::FormId;
use crate::handler::model::project::ProjectId;

use mime::Mime;
use serde::{Deserialize, Serialize};
use sos21_use_case::model::file_sharing as use_case;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileSharingId(pub Uuid);

impl FileSharingId {
    pub fn from_use_case(id: use_case::FileSharingId) -> FileSharingId {
        FileSharingId(id.0)
    }

    pub fn into_use_case(self) -> use_case::FileSharingId {
        use_case::FileSharingId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum FileSharingScope {
    Project {
        id: ProjectId,
    },
    FormAnswer {
        project_id: ProjectId,
        form_id: FormId,
    },
    Committee,
    CommitteeOperator,
    Public,
}

impl FileSharingScope {
    pub fn from_use_case(scope: use_case::FileSharingScope) -> Self {
        match scope {
            use_case::FileSharingScope::Project(project_id) => FileSharingScope::Project {
                id: ProjectId::from_use_case(project_id),
            },
            use_case::FileSharingScope::FormAnswer(project_id, form_id) => {
                FileSharingScope::FormAnswer {
                    project_id: ProjectId::from_use_case(project_id),
                    form_id: FormId::from_use_case(form_id),
                }
            }
            use_case::FileSharingScope::Committee => FileSharingScope::Committee,
            use_case::FileSharingScope::CommitteeOperator => FileSharingScope::CommitteeOperator,
            use_case::FileSharingScope::Public => FileSharingScope::Public,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSharing {
    pub id: FileSharingId,
    pub created_at: DateTime,
    pub is_revoked: bool,
    pub expires_at: Option<DateTime>,
    pub scope: FileSharingScope,
    pub file_id: FileId,
    pub file_name: Option<String>,
    #[serde(with = "crate::handler::model::serde::mime")]
    pub file_type: Mime,
    pub file_size: u64,
}

impl FileSharing {
    pub fn from_use_case(sharing: use_case::FileSharing) -> Self {
        FileSharing {
            id: FileSharingId::from_use_case(sharing.id),
            created_at: DateTime::from_use_case(sharing.created_at),
            is_revoked: sharing.is_revoked,
            expires_at: sharing.expires_at.map(DateTime::from_use_case),
            scope: FileSharingScope::from_use_case(sharing.scope),
            file_id: FileId::from_use_case(sharing.file_id),
            file_name: sharing.file_name,
            file_type: sharing.file_type,
            file_size: sharing.file_size,
        }
    }
}
