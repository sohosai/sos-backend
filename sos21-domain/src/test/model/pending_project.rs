use crate::model::{
    date_time::DateTime,
    pending_project::{PendingProject, PendingProjectContent, PendingProjectId},
    project::{ProjectAttribute, ProjectAttributes, ProjectCategory},
    user::UserId,
};
use crate::test::model as test_model;

use uuid::Uuid;

pub fn new_pending_project_id() -> PendingProjectId {
    PendingProjectId::from_uuid(Uuid::new_v4())
}

/// # Panics
///
/// This function panics when `attributes` contains duplicated elements.
pub fn new_pending_project_with_attributes(
    author_id: UserId,
    category: ProjectCategory,
    attributes: &[ProjectAttribute],
) -> PendingProject {
    PendingProject::from_content(
        PendingProjectContent {
            id: new_pending_project_id(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            name: test_model::mock_project_name(),
            kana_name: test_model::mock_project_kana_name(),
            group_name: test_model::mock_project_group_name(),
            kana_group_name: test_model::mock_project_kana_group_name(),
            description: test_model::mock_project_description(),
            category,
            attributes: ProjectAttributes::from_attributes(attributes.iter().copied()).unwrap(),
        },
        author_id,
    )
}

pub fn new_pending_project(author_id: UserId, category: ProjectCategory) -> PendingProject {
    new_pending_project_with_attributes(author_id, category, &[])
}

pub fn new_general_pending_project(author_id: UserId) -> PendingProject {
    new_pending_project(author_id, ProjectCategory::General)
}

pub fn new_stage_pending_project(author_id: UserId) -> PendingProject {
    new_pending_project(author_id, ProjectCategory::Stage)
}
