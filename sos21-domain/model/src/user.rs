use crate::email::EmailAddress;
use crate::permissions::Permissions;

use chrono::{DateTime, Utc};
use thiserror::Error;

mod name;
mod role;
pub use name::{UserKanaName, UserName};
pub use role::UserRole;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(pub String);

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub created_at: DateTime<Utc>,
    pub name: UserName,
    pub kana_name: UserKanaName,
    pub email: EmailAddress,
    pub role: UserRole,
}

#[derive(Debug, Error, Clone)]
#[error("insufficient permissions")]
pub struct RequirePermissionsError {
    _priv: (),
}

impl User {
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
}

#[cfg(test)]
mod tests {
    use sos21_domain_test::model as test_model;

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
