use crate::model::{
    date_time::DateTime,
    file_distribution::{
        FileDistribution, FileDistributionDescription, FileDistributionFiles, FileDistributionId,
        FileDistributionName,
    },
    file_sharing::{FileSharing, FileSharingId},
    user::UserId,
};
use crate::test::model as test_model;

use uuid::Uuid;

pub fn new_file_distribution_id() -> FileDistributionId {
    FileDistributionId::from_uuid(Uuid::new_v4())
}

pub fn mock_file_distribution_name() -> FileDistributionName {
    FileDistributionName::from_string("テスト配布").unwrap()
}

pub fn mock_file_distribution_description() -> FileDistributionDescription {
    FileDistributionDescription::from_string("テストテスト").unwrap()
}

/// # Panics
///
/// This function panics when the given `sharing` has a scope other than
/// `FileSharingScope::Project`.
pub fn mock_file_distribution_files_with_project_sharing(
    sharing: &FileSharing,
) -> FileDistributionFiles {
    let project_id = sharing.scope().project().unwrap();
    FileDistributionFiles::from_sharings(vec![(project_id, sharing.id())]).unwrap()
}

pub fn mock_file_distribution_files() -> FileDistributionFiles {
    FileDistributionFiles::from_sharings(vec![(
        test_model::new_project_id(),
        FileSharingId::from_uuid(Uuid::new_v4()),
    )])
    .unwrap()
}

pub fn new_file_distribution_with_files(
    author_id: UserId,
    files: FileDistributionFiles,
) -> FileDistribution {
    FileDistribution {
        id: new_file_distribution_id(),
        created_at: DateTime::now(),
        author_id,
        name: mock_file_distribution_name(),
        description: mock_file_distribution_description(),
        files,
    }
}

pub fn new_file_distribution(author_id: UserId) -> FileDistribution {
    new_file_distribution_with_files(author_id, mock_file_distribution_files())
}
