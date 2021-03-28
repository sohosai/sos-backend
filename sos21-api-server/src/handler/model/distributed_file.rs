use crate::handler::model::date_time::DateTime;
use crate::handler::model::file_distribution::FileDistributionId;
use crate::handler::model::file_sharing::FileSharingId;
use crate::handler::model::project::ProjectId;

use serde::{Deserialize, Serialize};
use sos21_use_case::model::file_distribution as use_case;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedFile {
    pub distribution_id: FileDistributionId,
    pub distributed_at: DateTime,
    pub name: String,
    pub description: String,
    pub project_id: ProjectId,
    pub sharing_id: FileSharingId,
}

impl DistributedFile {
    pub fn from_use_case(distributed_file: use_case::FileDistributionDistributedFile) -> Self {
        DistributedFile {
            distribution_id: FileDistributionId::from_use_case(distributed_file.distribution_id),
            distributed_at: DateTime::from_use_case(distributed_file.distributed_at),
            name: distributed_file.name,
            description: distributed_file.description,
            project_id: ProjectId::from_use_case(distributed_file.project_id),
            sharing_id: FileSharingId::from_use_case(distributed_file.sharing_id),
        }
    }
}
