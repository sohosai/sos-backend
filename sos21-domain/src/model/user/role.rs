use crate::model::permissions::Permissions;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Administrator,
    CommitteeOperator,
    Committee,
    General,
}

#[derive(Debug, Error, Clone)]
#[error("insufficient permissions")]
pub struct RequirePermissionsError {
    _priv: (),
}

impl UserRole {
    pub fn permissions(&self) -> Permissions {
        match self {
            UserRole::Administrator => Permissions::all(),
            UserRole::CommitteeOperator => {
                UserRole::Committee.permissions()
                    | Permissions::READ_ALL_USERS
                    | Permissions::CREATE_FORMS
            }
            UserRole::Committee => {
                UserRole::General.permissions()
                    | Permissions::READ_ALL_PROJECTS
                    | Permissions::READ_ALL_FORMS
                    | Permissions::READ_ALL_FORM_ANSWERS
            }
            UserRole::General => Permissions::CREATE_FILES,
        }
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

    pub fn file_usage_quota(&self) -> Option<u64> {
        match self {
            UserRole::General | UserRole::Committee => Some(256 * 1024 * 1024),
            UserRole::CommitteeOperator => Some(1024 * 1024 * 1024),
            UserRole::Administrator => None,
        }
    }
}
