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
pub struct UserContent {
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

#[derive(Debug, Clone)]
pub struct User {
    content: UserContent,
}

#[derive(Debug, Error, Clone)]
#[error("insufficient permissions")]
pub struct RequirePermissionsError {
    _priv: (),
}

impl User {
    /// Restore `User` from `UserContent`.
    ///
    /// This is intended to be used when the data is taken out of the implementation
    /// by [`User::into_content`] for persistence, internal serialization, etc.
    /// Use [`User::new`] to create a project.
    pub fn from_content(content: UserContent) -> Self {
        User { content }
    }

    /// Convert `User` into `UserContent`.
    pub fn into_content(self) -> UserContent {
        self.content
    }

    pub fn id(&self) -> &UserId {
        &self.content.id
    }

    pub fn created_at(&self) -> DateTime {
        self.content.created_at
    }

    pub fn name(&self) -> &UserName {
        &self.content.name
    }

    pub fn kana_name(&self) -> &UserKanaName {
        &self.content.kana_name
    }

    pub fn phone_number(&self) -> &PhoneNumber {
        &self.content.phone_number
    }

    pub fn affiliation(&self) -> &UserAffiliation {
        &self.content.affiliation
    }

    pub fn email(&self) -> &UserEmailAddress {
        &self.content.email
    }

    pub fn role(&self) -> UserRole {
        self.content.role
    }

    pub fn category(&self) -> UserCategory {
        self.content.category
    }

    pub fn assignment(&self) -> Option<UserAssignment> {
        self.content.assignment
    }

    pub fn assign_project_owner(&mut self, project: &Project) -> anyhow::Result<()> {
        anyhow::ensure!(project.owner_id() == self.id());
        self.content
            .assignment
            .replace(UserAssignment::ProjectOwner(project.id()));
        Ok(())
    }

    pub fn assign_project_subowner(&mut self, project: &Project) -> anyhow::Result<()> {
        anyhow::ensure!(project.subowner_id() == self.id());
        self.content
            .assignment
            .replace(UserAssignment::ProjectSubowner(project.id()));
        Ok(())
    }

    pub fn assign_pending_project_owner(
        &mut self,
        pending_project: &PendingProject,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(pending_project.owner_id() == self.id());
        self.content
            .assignment
            .replace(UserAssignment::PendingProjectOwner(pending_project.id()));
        Ok(())
    }

    pub fn permissions(&self) -> Permissions {
        self.role().permissions()
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
        if self.id() == user.id() {
            return true;
        }

        user.permissions().contains(Permissions::READ_ALL_USERS)
    }

    pub async fn file_usage<C>(&self, ctx: C) -> anyhow::Result<UserFileUsage>
    where
        C: FileRepository,
    {
        ctx.sum_file_usage_by_user(self.id().clone())
            .await
            .context("Failed to sum usage by user")
    }

    pub fn file_usage_quota(&self) -> UserFileUsageQuota {
        self.role().file_usage_quota()
    }

    pub fn is_committee(&self) -> bool {
        self.role().is_committee()
    }

    pub fn is_committee_operator(&self) -> bool {
        self.role().is_committee_operator()
    }

    pub fn set_name(&mut self, name: UserName) {
        self.content.name = name;
    }

    pub fn set_kana_name(&mut self, kana_name: UserKanaName) {
        self.content.kana_name = kana_name;
    }

    pub fn set_phone_number(&mut self, phone_number: PhoneNumber) {
        self.content.phone_number = phone_number;
    }

    pub fn set_affiliation(&mut self, affiliation: UserAffiliation) {
        self.content.affiliation = affiliation;
    }

    pub fn set_role(&mut self, role: UserRole) {
        self.content.role = role;
    }

    pub fn set_category(&mut self, category: UserCategory) {
        self.content.category = category;
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
