use crate::model::date_time::DateTime;
use crate::model::file::{File, FileId};
use crate::model::form_answer::FormAnswer;
use crate::model::project::Project;
use crate::model::user::User;

use thiserror::Error;
use uuid::Uuid;

mod scope;
pub use scope::FileSharingScope;
mod state;
pub use state::FileSharingState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileSharingId(Uuid);

impl FileSharingId {
    pub fn from_uuid(uuid: Uuid) -> FileSharingId {
        FileSharingId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct FileSharingContent {
    pub id: FileSharingId,
    pub created_at: DateTime,
    pub file_id: FileId,
    pub is_revoked: bool,
    pub expires_at: Option<DateTime>,
    pub scope: FileSharingScope,
}

#[derive(Debug, Clone)]
pub struct FileSharing(FileSharingContent);

#[derive(Debug, Error, Clone)]
#[error("invalid expiration date")]
pub struct InvalidExpirationDateError {
    _priv: (),
}

#[derive(Debug, Clone, Copy)]
pub enum RevokeErrorKind {
    AlreadyRevokedSharing,
    ExpiredSharing,
}

#[derive(Debug, Error, Clone)]
#[error("failed to revoke the sharing")]
pub struct RevokeError {
    kind: RevokeErrorKind,
}

impl RevokeError {
    pub fn kind(&self) -> RevokeErrorKind {
        self.kind
    }
}

#[derive(Debug, Clone)]
pub struct FileSharingWitness(FileId);

impl FileSharingWitness {
    pub fn file_id(&self) -> FileId {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ToWitnessErrorKind {
    OutOfScope,
    RevokedSharing,
    ExpiredSharing,
}

#[derive(Debug, Error, Clone)]
#[error("failed to validate the sharing")]
pub struct ToWitnessError {
    kind: ToWitnessErrorKind,
}

impl ToWitnessError {
    pub fn kind(&self) -> ToWitnessErrorKind {
        self.kind
    }
}

impl FileSharing {
    pub fn new(file_id: FileId, scope: FileSharingScope) -> Self {
        FileSharing(FileSharingContent {
            id: FileSharingId::from_uuid(Uuid::new_v4()),
            created_at: DateTime::now(),
            file_id,
            is_revoked: false,
            expires_at: None,
            scope,
        })
    }

    pub fn with_expiration(
        file_id: FileId,
        scope: FileSharingScope,
        expires_at: DateTime,
    ) -> Result<Self, InvalidExpirationDateError> {
        let created_at = DateTime::now();
        if expires_at <= created_at {
            return Err(InvalidExpirationDateError { _priv: () });
        }

        Ok(FileSharing(FileSharingContent {
            id: FileSharingId::from_uuid(Uuid::new_v4()),
            created_at,
            file_id,
            is_revoked: false,
            expires_at: Some(expires_at),
            scope,
        }))
    }

    pub fn from_content(content: FileSharingContent) -> Self {
        FileSharing(content)
    }

    pub fn into_content(self) -> FileSharingContent {
        self.0
    }

    pub fn id(&self) -> FileSharingId {
        self.0.id
    }

    pub fn file_id(&self) -> FileId {
        self.0.file_id
    }

    /// Equivalent to `sharing.state_at(DateTime::now())`
    pub fn state(&self) -> FileSharingState {
        self.state_at(DateTime::now())
    }

    pub fn state_at(&self, now: DateTime) -> FileSharingState {
        if let Some(expires_at) = self.0.expires_at {
            if expires_at <= now {
                return FileSharingState::Expired;
            }
        }

        if self.is_revoked() {
            FileSharingState::Revoked
        } else {
            FileSharingState::Active
        }
    }

    /// Check if the sharing is revoked or not, independently from `DateTime::now`
    pub fn is_revoked(&self) -> bool {
        self.0.is_revoked
    }

    pub fn revoke(&mut self) -> Result<(), RevokeError> {
        match self.state() {
            FileSharingState::Revoked => Err(RevokeError {
                kind: RevokeErrorKind::AlreadyRevokedSharing,
            }),
            FileSharingState::Expired => Err(RevokeError {
                kind: RevokeErrorKind::ExpiredSharing,
            }),
            FileSharingState::Active => {
                self.0.is_revoked = true;
                Ok(())
            }
        }
    }

    pub fn is_visible_to(&self, _user: &User) -> bool {
        false
    }

    pub fn is_visible_to_with_file(&self, user: &User, file: &File) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.0.file_id == file.id && file.is_visible_to(user)
    }

    fn state_to_witness(&self) -> Result<FileSharingWitness, ToWitnessError> {
        match self.state() {
            FileSharingState::Revoked => Err(ToWitnessError {
                kind: ToWitnessErrorKind::RevokedSharing,
            }),
            FileSharingState::Expired => Err(ToWitnessError {
                kind: ToWitnessErrorKind::ExpiredSharing,
            }),
            FileSharingState::Active => Ok(FileSharingWitness(self.0.file_id)),
        }
    }

    pub fn to_witness(&self) -> Result<FileSharingWitness, ToWitnessError> {
        if !self.0.scope.is_public() {
            return Err(ToWitnessError {
                kind: ToWitnessErrorKind::OutOfScope,
            });
        }

        self.state_to_witness()
    }

    pub fn to_witness_with_user(&self, user: &User) -> Result<FileSharingWitness, ToWitnessError> {
        if !self.0.scope.contains_user(user) {
            return Err(ToWitnessError {
                kind: ToWitnessErrorKind::OutOfScope,
            });
        }

        self.state_to_witness()
    }

    pub fn to_witness_with_project(
        &self,
        project: &Project,
    ) -> Result<FileSharingWitness, ToWitnessError> {
        if !self.0.scope.contains_project(project) {
            return Err(ToWitnessError {
                kind: ToWitnessErrorKind::OutOfScope,
            });
        }

        self.state_to_witness()
    }

    pub fn to_witness_with_form_answer(
        &self,
        answer: &FormAnswer,
    ) -> Result<FileSharingWitness, ToWitnessError> {
        if !self.0.scope.contains_form_answer(answer) {
            return Err(ToWitnessError {
                kind: ToWitnessErrorKind::OutOfScope,
            });
        }

        self.state_to_witness()
    }
}

#[cfg(test)]
mod tests {
    use super::{FileSharing, FileSharingScope};
    use crate::model::date_time::DateTime;
    use crate::test::model as test_model;

    #[test]
    fn test_new_past_expiration() {
        use super::InvalidExpirationDateError;

        let expires_at = DateTime::from_utc(chrono::Utc::now() - chrono::Duration::days(1));
        let file_id = test_model::new_file_id();
        assert!(matches!(
            FileSharing::with_expiration(file_id, FileSharingScope::Public, expires_at),
            Err(InvalidExpirationDateError { .. })
        ));
    }

    #[test]
    fn test_revoke_revoked() {
        use super::RevokeErrorKind;

        let mut sharing = FileSharing::new(test_model::new_file_id(), FileSharingScope::Committee);
        sharing.revoke().unwrap();
        assert!(matches!(
            sharing.revoke().unwrap_err().kind(),
            RevokeErrorKind::AlreadyRevokedSharing
        ));
    }

    #[test]
    fn test_revoke_expired() {
        use super::RevokeErrorKind;

        let mut sharing = test_model::new_expired_file_sharing(
            test_model::new_file_id(),
            FileSharingScope::CommitteeOperator,
        );
        assert!(matches!(
            sharing.revoke().unwrap_err().kind(),
            RevokeErrorKind::ExpiredSharing
        ));
    }

    #[test]
    fn test_visibility_direct() {
        let sharing = FileSharing::new(test_model::new_file_id(), FileSharingScope::Public);
        let user = test_model::new_admin_user();
        assert!(!sharing.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_with_owner_file() {
        let user = test_model::new_general_user();
        let (file, _) = test_model::new_file(user.id.clone());
        let sharing = FileSharing::new(file.id, FileSharingScope::Public);
        assert!(sharing.is_visible_to_with_file(&user, &file));
    }

    #[test]
    fn test_visibility_with_other_file() {
        let user = test_model::new_general_user();
        let other = test_model::new_general_user();
        let (file, _) = test_model::new_file(user.id.clone());
        let (other_file, _) = test_model::new_file(other.id.clone());
        let sharing = FileSharing::new(file.id, FileSharingScope::Public);
        assert!(!sharing.is_visible_to_with_file(&user, &other_file));
    }

    #[test]
    fn test_witness_revoked() {
        use super::ToWitnessErrorKind;

        let mut sharing = FileSharing::new(test_model::new_file_id(), FileSharingScope::Public);
        sharing.revoke().unwrap();
        assert!(matches!(
            sharing.to_witness().unwrap_err().kind(),
            ToWitnessErrorKind::RevokedSharing
        ));
    }

    #[test]
    fn test_witness_expired() {
        use super::ToWitnessErrorKind;

        let sharing = test_model::new_expired_file_sharing(
            test_model::new_file_id(),
            FileSharingScope::Public,
        );
        assert!(matches!(
            sharing.to_witness().unwrap_err().kind(),
            ToWitnessErrorKind::ExpiredSharing
        ));
    }

    #[test]
    fn test_witness_public_non_public() {
        use super::ToWitnessErrorKind;

        let sharing = FileSharing::new(test_model::new_file_id(), FileSharingScope::Committee);
        assert!(matches!(
            sharing.to_witness().unwrap_err().kind(),
            ToWitnessErrorKind::OutOfScope
        ));
    }

    #[test]
    fn test_witness_with_user_committee_operator() {
        use super::ToWitnessErrorKind;

        let user = test_model::new_committee_user();
        let sharing = FileSharing::new(
            test_model::new_file_id(),
            FileSharingScope::CommitteeOperator,
        );
        assert!(matches!(
            sharing.to_witness_with_user(&user).unwrap_err().kind(),
            ToWitnessErrorKind::OutOfScope
        ));
    }

    #[test]
    fn test_witness_with_user_operator_committee() {
        let user = test_model::new_operator_user();
        let sharing = FileSharing::new(test_model::new_file_id(), FileSharingScope::Committee);
        assert!(sharing.to_witness_with_user(&user).is_ok());
    }

    #[test]
    fn test_witness_with_project_out_of_scope() {
        use super::ToWitnessErrorKind;

        let sharing = FileSharing::new(
            test_model::new_file_id(),
            FileSharingScope::Project(test_model::new_project_id()),
        );
        let project = test_model::new_general_project(test_model::new_user_id());
        assert!(matches!(
            sharing
                .to_witness_with_project(&project)
                .unwrap_err()
                .kind(),
            ToWitnessErrorKind::OutOfScope
        ));
    }

    #[test]
    fn test_witness_with_form_answer_out_of_scope() {
        use super::ToWitnessErrorKind;

        let sharing = FileSharing::new(
            test_model::new_file_id(),
            FileSharingScope::FormAnswer(test_model::new_project_id(), test_model::new_form_id()),
        );
        let form = test_model::new_form(test_model::new_user_id());
        let answer = test_model::new_form_answer(
            test_model::new_user_id(),
            test_model::new_project_id(),
            &form,
        );
        assert!(matches!(
            sharing
                .to_witness_with_form_answer(&answer)
                .unwrap_err()
                .kind(),
            ToWitnessErrorKind::OutOfScope
        ));
    }
}
