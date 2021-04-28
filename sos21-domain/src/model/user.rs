use crate::context::FileRepository;
use crate::model::date_time::DateTime;
use crate::model::pending_project::PendingProject;
use crate::model::permissions::Permissions;
use crate::model::phone_number::PhoneNumber;
use crate::model::project::Project;

use anyhow::Context;
use thiserror::Error;

pub mod affiliation;
pub mod assignment;
pub mod category;
pub mod email;
pub mod file_usage;
pub mod file_usage_quota;
pub mod name;
pub mod role;
pub use affiliation::UserAffiliation;
pub use assignment::UserAssignment;
pub use category::UserCategory;
pub use email::UserEmailAddress;
pub use file_usage::UserFileUsage;
pub use file_usage_quota::UserFileUsageQuota;
pub use name::{UserKanaName, UserName};
pub use role::UserRole;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(pub String);

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub created_at: DateTime,
    pub name: UserName,
    pub kana_name: UserKanaName,
    pub phone_number: PhoneNumber,
    pub affiliation: UserAffiliation,
    pub email: UserEmailAddress,
    pub role: UserRole,
    pub category: UserCategory,
    pub assignment: Option<UserAssignment>,
}

#[derive(Debug, Error, Clone)]
#[error("insufficient permissions")]
pub struct RequirePermissionsError {
    _priv: (),
}

impl User {
    pub fn assignment(&self) -> Option<UserAssignment> {
        self.assignment
    }

    pub fn assign_project_owner(&mut self, project: &Project) -> anyhow::Result<()> {
        anyhow::ensure!(project.owner_id() == &self.id);
        self.assignment
            .replace(UserAssignment::ProjectOwner(project.id()));
        Ok(())
    }

    pub fn assign_project_subowner(&mut self, project: &Project) -> anyhow::Result<()> {
        anyhow::ensure!(project.subowner_id() == &self.id);
        self.assignment
            .replace(UserAssignment::ProjectSubowner(project.id()));
        Ok(())
    }

    pub fn assign_pending_project_owner(
        &mut self,
        pending_project: &PendingProject,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(pending_project.owner_id() == &self.id);
        self.assignment
            .replace(UserAssignment::PendingProjectOwner(pending_project.id()));
        Ok(())
    }

    pub fn permissions(&self) -> Permissions {
        self.role.permissions()
    }

    pub fn require_permissions(
        &self,
        permissions: Permissions,
    ) -> Result<(), RequirePermissionsError> {
        if self.permissions().contains(permissions) {
            Ok(())
        } else {
            Err(RequirePermissionsError { _priv: () })
        }
    }

    pub fn is_visible_to(&self, user: &User) -> bool {
        if self.id == user.id {
            return true;
        }

        user.permissions().contains(Permissions::READ_ALL_USERS)
    }

    pub async fn file_usage<C>(&self, ctx: C) -> anyhow::Result<UserFileUsage>
    where
        C: FileRepository,
    {
        ctx.sum_file_usage_by_user(self.id.clone())
            .await
            .context("Failed to sum usage by user")
    }

    pub fn file_usage_quota(&self) -> UserFileUsageQuota {
        self.role.file_usage_quota()
    }

    pub fn is_committee(&self) -> bool {
        self.role.is_committee()
    }

    pub fn is_committee_operator(&self) -> bool {
        self.role.is_committee_operator()
    }

    pub fn set_name(&mut self, name: UserName) {
        self.name = name;
    }

    pub fn set_kana_name(&mut self, kana_name: UserKanaName) {
        self.kana_name = kana_name;
    }

    pub fn set_phone_number(&mut self, phone_number: PhoneNumber) {
        self.phone_number = phone_number;
    }

    pub fn set_affiliation(&mut self, affiliation: UserAffiliation) {
        self.affiliation = affiliation;
    }

    pub fn set_role(&mut self, role: UserRole) {
        self.role = role;
    }

    pub fn set_category(&mut self, category: UserCategory) {
        self.category = category;
    }
}

#[cfg(test)]
mod tests {
    use crate::test::model as test_model;

    #[test]
    fn test_visibility_general_self() {
        let user = test_model::new_general_user();
        assert!(user.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_other() {
        let user = test_model::new_general_user();
        let other = test_model::new_general_user();
        assert!(!other.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee_other() {
        let user = test_model::new_committee_user();
        let other = test_model::new_general_user();
        assert!(!other.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator_other() {
        let user = test_model::new_operator_user();
        let other = test_model::new_general_user();
        assert!(other.is_visible_to(&user));
    }
}
