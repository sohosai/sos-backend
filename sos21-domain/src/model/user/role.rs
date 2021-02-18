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
                Permissions::READ_ALL_USERS
                    | Permissions::READ_ALL_PROJECTS
                    | Permissions::READ_ALL_FORMS
                    | Permissions::CREATE_FORMS
            }
            UserRole::Committee => Permissions::READ_ALL_PROJECTS | Permissions::READ_ALL_FORMS,
            UserRole::General => Permissions::empty(),
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
}
