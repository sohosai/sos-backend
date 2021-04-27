use std::sync::atomic::{AtomicU16, Ordering};

use crate::model::{
    date_time::DateTime,
    project::{
        Project, ProjectAttribute, ProjectAttributes, ProjectCategory, ProjectContent,
        ProjectDescription, ProjectGroupName, ProjectId, ProjectIndex, ProjectKanaGroupName,
        ProjectKanaName, ProjectName,
    },
    user::UserId,
};
use crate::test::model as test_model;

use once_cell::sync::OnceCell;
use uuid::Uuid;

pub fn new_project_id() -> ProjectId {
    ProjectId::from_uuid(Uuid::new_v4())
}

static NEXT_PROJECT_INDEX: OnceCell<AtomicU16> = OnceCell::new();

pub fn new_project_index() -> ProjectIndex {
    let index = NEXT_PROJECT_INDEX.get_or_init(|| AtomicU16::new(0));
    let index = index.fetch_add(1, Ordering::SeqCst);
    ProjectIndex::from_u16(index).unwrap()
}

pub fn mock_project_name() -> ProjectName {
    ProjectName::from_string("mock プロジェクト").unwrap()
}

pub fn mock_project_kana_name() -> ProjectKanaName {
    ProjectKanaName::from_string("モック プロジェクト").unwrap()
}

pub fn mock_project_group_name() -> ProjectGroupName {
    ProjectGroupName::from_string("jsys20").unwrap()
}

pub fn mock_project_kana_group_name() -> ProjectKanaGroupName {
    ProjectKanaGroupName::from_string("じょーしすにじゅう").unwrap()
}

pub fn mock_project_description() -> ProjectDescription {
    ProjectDescription::from_string("これはテスト用のモックデータです。").unwrap()
}

/// # Panics
///
/// This function panics when `attributes` contains duplicated elements.
pub fn new_project_with_attributes(
    owner_id: UserId,
    category: ProjectCategory,
    attributes: &[ProjectAttribute],
) -> Project {
    Project::from_content(ProjectContent {
        id: new_project_id(),
        index: new_project_index(),
        created_at: DateTime::now(),
        owner_id,
        subowner_id: test_model::KNOWN_MOCK_GENERAL_USER_ID.clone(),
        name: mock_project_name(),
        kana_name: mock_project_kana_name(),
        group_name: mock_project_group_name(),
        kana_group_name: mock_project_kana_group_name(),
        description: mock_project_description(),
        category,
        attributes: ProjectAttributes::from_attributes(attributes.iter().copied()).unwrap(),
    })
    .unwrap()
}

pub fn new_project_with_subowner(
    owner_id: UserId,
    subowner_id: UserId,
    category: ProjectCategory,
) -> Project {
    Project::from_content(ProjectContent {
        id: new_project_id(),
        index: new_project_index(),
        created_at: DateTime::now(),
        owner_id,
        subowner_id,
        name: mock_project_name(),
        kana_name: mock_project_kana_name(),
        group_name: mock_project_group_name(),
        kana_group_name: mock_project_kana_group_name(),
        description: mock_project_description(),
        category,
        attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
    })
    .unwrap()
}

pub fn new_project(owner_id: UserId, category: ProjectCategory) -> Project {
    new_project_with_attributes(owner_id, category, &[])
}

pub fn new_general_project(owner_id: UserId) -> Project {
    new_project(owner_id, ProjectCategory::General)
}

pub fn new_stage_project(owner_id: UserId) -> Project {
    new_project(owner_id, ProjectCategory::Stage)
}

pub fn new_general_project_with_subowner(owner_id: UserId, subowner_id: UserId) -> Project {
    new_project_with_subowner(owner_id, subowner_id, ProjectCategory::General)
}

pub fn new_stage_project_with_subowner(owner_id: UserId, subowner_id: UserId) -> Project {
    new_project_with_subowner(owner_id, subowner_id, ProjectCategory::Stage)
}
