use sos21_domain_model::{
    date_time::DateTime,
    project::{
        Project, ProjectAttribute, ProjectAttributes, ProjectCategory, ProjectDescription,
        ProjectDisplayId, ProjectGroupName, ProjectId, ProjectKanaGroupName, ProjectKanaName,
        ProjectName,
    },
    user::UserId,
};
use uuid::Uuid;

pub fn new_project_id() -> ProjectId {
    ProjectId(Uuid::new_v4())
}

pub fn mock_project_display_id() -> ProjectDisplayId {
    ProjectDisplayId::from_string("mock_project_id").unwrap()
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
    Project {
        id: new_project_id(),
        created_at: DateTime::now(),
        display_id: mock_project_display_id(),
        owner_id,
        name: mock_project_name(),
        kana_name: mock_project_kana_name(),
        group_name: mock_project_group_name(),
        kana_group_name: mock_project_kana_group_name(),
        description: mock_project_description(),
        category,
        attributes: ProjectAttributes::from_attributes(attributes.iter().copied()).unwrap(),
    }
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
