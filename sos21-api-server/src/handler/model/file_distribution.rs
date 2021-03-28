use crate::handler::model::date_time::DateTime;
use crate::handler::model::file_sharing::FileSharingId;
use crate::handler::model::project::ProjectId;
use crate::handler::model::user::UserId;

use serde::{Deserialize, Serialize};
use sos21_use_case::model::file_distribution as use_case;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileDistributionId(pub Uuid);

impl FileDistributionId {
    pub fn from_use_case(id: use_case::FileDistributionId) -> Self {
        FileDistributionId(id.0)
    }

    pub fn into_use_case(self) -> use_case::FileDistributionId {
        use_case::FileDistributionId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDistributionFileMapping {
    pub project_id: ProjectId,
    pub sharing_id: FileSharingId,
}

impl FileDistributionFileMapping {
    pub fn from_use_case(mapping: use_case::FileDistributionFileMapping) -> Self {
        FileDistributionFileMapping {
            project_id: ProjectId::from_use_case(mapping.project_id),
            sharing_id: FileSharingId::from_use_case(mapping.sharing_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDistribution {
    pub id: FileDistributionId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub name: String,
    pub description: String,
    pub files: Vec<FileDistributionFileMapping>,
}

impl FileDistribution {
    pub fn from_use_case(distribution: use_case::FileDistribution) -> Self {
        FileDistribution {
            id: FileDistributionId::from_use_case(distribution.id),
            created_at: DateTime::from_use_case(distribution.created_at),
            author_id: UserId::from_use_case(distribution.author_id),
            name: distribution.name,
            description: distribution.description,
            files: distribution
                .files
                .into_iter()
                .map(FileDistributionFileMapping::from_use_case)
                .collect(),
        }
    }
}
