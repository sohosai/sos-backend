use crate::model::{
    date_time::DateTime,
    pending_project::{PendingProject, PendingProjectId},
    project::{ProjectAttributes, ProjectCategory},
    user::UserId,
};
use crate::test::model as test_model;

use uuid::Uuid;

pub fn new_pending_project_id() -> PendingProjectId {
    PendingProjectId::from_uuid(Uuid::new_v4())
}

pub fn new_pending_project(author_id: UserId) -> PendingProject {
    PendingProject {
        id: new_pending_project_id(),
        created_at: DateTime::now(),
        author_id,
        name: test_model::mock_project_name(),
        kana_name: test_model::mock_project_kana_name(),
        group_name: test_model::mock_project_group_name(),
        kana_group_name: test_model::mock_project_kana_group_name(),
        description: test_model::mock_project_description(),
        category: ProjectCategory::General,
        attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
    }
}
