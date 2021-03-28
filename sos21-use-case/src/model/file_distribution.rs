use crate::model::file_sharing::FileSharingId;
use crate::model::project::ProjectId;
use crate::model::user::UserId;

use chrono::{DateTime, Utc};
use sos21_domain::model::file_distribution as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDistributionId(pub Uuid);

impl FileDistributionId {
    pub fn from_entity(id: entity::FileDistributionId) -> FileDistributionId {
        FileDistributionId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::FileDistributionId {
        entity::FileDistributionId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct FileDistributionDistributedFile {
    pub distribution_id: FileDistributionId,
    pub distributed_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
    pub project_id: ProjectId,
    pub sharing_id: FileSharingId,
}

impl FileDistributionDistributedFile {
    pub fn from_entity(distributed_file: entity::FileDistributionDistributedFile) -> Self {
        FileDistributionDistributedFile {
            distribution_id: FileDistributionId::from_entity(distributed_file.distribution_id),
            distributed_at: distributed_file.distributed_at.utc(),
            name: distributed_file.name.into_string(),
            description: distributed_file.description.into_string(),
            project_id: ProjectId::from_entity(distributed_file.project_id),
            sharing_id: FileSharingId::from_entity(distributed_file.sharing_id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileDistribution {
    pub id: FileDistributionId,
    pub created_at: DateTime<Utc>,
    pub author_id: UserId,
    pub name: String,
    pub description: String,
    pub files: Vec<FileDistributionFileMapping>,
}

#[derive(Debug, Clone)]
pub struct FileDistributionFileMapping {
    pub project_id: ProjectId,
    pub sharing_id: FileSharingId,
}

impl FileDistribution {
    pub fn from_entity(distribution: entity::FileDistribution) -> Self {
        FileDistribution {
            id: FileDistributionId::from_entity(distribution.id),
            created_at: distribution.created_at.utc(),
            author_id: UserId::from_entity(distribution.author_id),
            name: distribution.name.into_string(),
            description: distribution.description.into_string(),
            files: distribution
                .files
                .sharings()
                .map(|(project_id, sharing_id)| FileDistributionFileMapping {
                    project_id: ProjectId::from_entity(project_id),
                    sharing_id: FileSharingId::from_entity(sharing_id),
                })
                .collect(),
        }
    }
}
