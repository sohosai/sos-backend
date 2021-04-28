use crate::model::date_time::DateTime;
use crate::model::permissions::Permissions;
use crate::model::project::Project;
use crate::model::user::{User, UserId};

use thiserror::Error;
use uuid::Uuid;

pub mod files;
pub use files::FileDistributionFiles;
pub mod distributed_file;
pub use distributed_file::FileDistributionDistributedFile;
pub mod description;
pub use description::FileDistributionDescription;
pub mod name;
pub use name::FileDistributionName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileDistributionId(Uuid);

impl FileDistributionId {
    pub fn from_uuid(uuid: Uuid) -> FileDistributionId {
        FileDistributionId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct FileDistribution {
    pub id: FileDistributionId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub name: FileDistributionName,
    pub description: FileDistributionDescription,
    pub files: FileDistributionFiles,
}

#[derive(Debug, Error, Clone)]
#[error("the project is not targeted by the distribution")]
pub struct NotTargetedError {
    _priv: (),
}

impl FileDistribution {
    pub fn is_visible_to(&self, user: &User) -> bool {
        user.permissions()
            .contains(Permissions::READ_ALL_FILE_DISTRIBUTIONS)
    }

    pub fn is_targeted_to(&self, project: &Project) -> bool {
        self.files.contains_project(project)
    }

    pub fn get_distributed_file_for(
        &self,
        project: &Project,
    ) -> Result<FileDistributionDistributedFile, NotTargetedError> {
        let sharing_id = match self.files.get_sharing_for(project) {
            Some(sharing_id) => sharing_id,
            None => {
                return Err(NotTargetedError { _priv: () });
            }
        };

        Ok(FileDistributionDistributedFile {
            distribution_id: self.id,
            distributed_at: self.created_at,
            name: self.name.clone(),
            description: self.description.clone(),
            project_id: project.id(),
            sharing_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::test::model as test_model;

    #[test]
    fn test_visibility_general() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let distribution = test_model::new_file_distribution(operator.id);
        assert!(!distribution.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee() {
        let user = test_model::new_committee_user();
        let operator = test_model::new_operator_user();
        let distribution = test_model::new_file_distribution(operator.id);
        assert!(distribution.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator() {
        let user = test_model::new_operator_user();
        let operator = test_model::new_operator_user();
        let distribution = test_model::new_file_distribution(operator.id);
        assert!(distribution.is_visible_to(&user));
    }
}
