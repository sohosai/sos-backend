use crate::model::permissions::Permissions;
use crate::model::user::UserFileUsageQuota;

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
                    | Permissions::UPDATE_NOT_STARTED_OWNING_FORMS
                    | Permissions::DISTRIBUTE_FILES
                    | Permissions::CREATE_REGISTRATION_FORMS
            }
            UserRole::Committee => {
                UserRole::General.permissions()
                    | Permissions::READ_ALL_PROJECTS
                    | Permissions::READ_ALL_FORMS
                    | Permissions::READ_ALL_FORM_ANSWERS
                    | Permissions::READ_ALL_FILE_DISTRIBUTIONS
                    | Permissions::READ_ALL_REGISTRATION_FORMS
                    | Permissions::READ_ALL_REGISTRATION_FORM_ANSWERS
            }
            UserRole::General => {
                Permissions::CREATE_FILES
                    | Permissions::SHARE_FILES
                    | Permissions::ANSWER_REGISTRATION_FORMS
                    | Permissions::UPDATE_FORM_ANSWERS_IN_PERIOD
                    | Permissions::UPDATE_MEMBER_PROJECTS_IN_PERIOD
                    | Permissions::UPDATE_OWNING_PENDING_PROJECTS_IN_PERIOD
            }
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

    pub fn file_usage_quota(&self) -> UserFileUsageQuota {
        match self {
            UserRole::General | UserRole::Committee => {
                UserFileUsageQuota::limited_number_of_bytes(512 * 1024 * 1024)
            }
            UserRole::CommitteeOperator => {
                UserFileUsageQuota::limited_number_of_bytes(15 * 1024 * 1024 * 1024)
            }
            UserRole::Administrator => {
                UserFileUsageQuota::limited_number_of_bytes(256 * 1024 * 1024 * 1024)
            }
        }
    }

    pub fn is_committee(&self) -> bool {
        matches!(self, UserRole::CommitteeOperator | UserRole::Committee)
    }

    pub fn is_committee_operator(&self) -> bool {
        matches!(self, UserRole::CommitteeOperator)
    }
}
