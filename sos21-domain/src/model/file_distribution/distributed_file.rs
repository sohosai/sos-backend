use crate::model::date_time::DateTime;
use crate::model::file_distribution::{
    FileDistributionDescription, FileDistributionId, FileDistributionName,
};
use crate::model::file_sharing::FileSharingId;
use crate::model::permissions::Permissions;
use crate::model::project::{Project, ProjectId};
use crate::model::user::User;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDistributionDistributedFile {
    pub distribution_id: FileDistributionId,
    pub distributed_at: DateTime,
    pub name: FileDistributionName,
    pub description: FileDistributionDescription,
    pub project_id: ProjectId,
    pub sharing_id: FileSharingId,
}

impl FileDistributionDistributedFile {
    pub fn is_visible_to(&self, user: &User) -> bool {
        user.permissions()
            .contains(Permissions::READ_ALL_FILE_DISTRIBUTIONS)
    }

    pub fn is_visible_to_with_project(&self, user: &User, project: &Project) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.project_id == project.id() && project.is_visible_to(user)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::file_distribution::FileDistributionFiles;
    use crate::model::file_sharing::FileSharingId;
    use crate::test::model as test_model;
    use uuid::Uuid;

    #[test]
    fn test_visibility_general() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let project = test_model::new_general_project(user.id().clone());
        let files = FileDistributionFiles::from_sharings(vec![(
            project.id(),
            FileSharingId::from_uuid(Uuid::new_v4()),
        )])
        .unwrap();
        let distribution =
            test_model::new_file_distribution_with_files(operator.id().clone(), files);
        let distributed_file = distribution.get_distributed_file_for(&project).unwrap();
        assert!(!distributed_file.is_visible_to(&user));
        assert!(distributed_file.is_visible_to_with_project(&user, &project));
    }

    #[test]
    fn test_visibility_committee() {
        let user = test_model::new_committee_user();
        let operator = test_model::new_operator_user();
        let project = test_model::new_general_project(user.id().clone());
        let files = FileDistributionFiles::from_sharings(vec![(
            project.id(),
            FileSharingId::from_uuid(Uuid::new_v4()),
        )])
        .unwrap();
        let distribution =
            test_model::new_file_distribution_with_files(operator.id().clone(), files);
        let distributed_file = distribution.get_distributed_file_for(&project).unwrap();
        assert!(distributed_file.is_visible_to(&user));
        assert!(distributed_file.is_visible_to_with_project(&user, &project));
    }
}
