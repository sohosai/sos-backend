use crate::model::date_time::DateTime;
use crate::model::file_sharing::{self, FileSharing, FileSharingScope, FileSharingWitness};
use crate::model::object::ObjectId;
use crate::model::user::{User, UserId};

use thiserror::Error;
use uuid::Uuid;

pub mod name;
pub use name::FileName;
pub mod type_;
pub use type_::FileType;
pub mod digest;
pub use digest::FileBlake3Digest;
pub mod size;
pub use size::FileSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(Uuid);

impl FileId {
    pub fn from_uuid(uuid: Uuid) -> FileId {
        FileId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub id: FileId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub object_id: ObjectId,
    pub blake3_digest: FileBlake3Digest,
    pub name: Option<FileName>,
    pub type_: FileType,
    pub size: FileSize,
}

#[derive(Debug, Error, Clone)]
#[error("the file cannot be shared by the user")]
pub struct NonSharableFileError {
    _priv: (),
}

#[derive(Debug, Clone, Copy)]
pub enum ShareWithExpirationErrorKind {
    NonSharableFile,
    InvalidExpirationDate,
}

#[derive(Debug, Error, Clone)]
#[error("the file cannot be shared")]
pub struct ShareWithExpirationError {
    kind: ShareWithExpirationErrorKind,
}

impl ShareWithExpirationError {
    pub fn kind(&self) -> ShareWithExpirationErrorKind {
        self.kind
    }

    fn from_sharing_expiration_error(_err: file_sharing::InvalidExpirationDateError) -> Self {
        ShareWithExpirationError {
            kind: ShareWithExpirationErrorKind::InvalidExpirationDate,
        }
    }
}

impl File {
    pub fn is_visible_to(&self, user: &User) -> bool {
        self.author_id == user.id
    }

    pub fn is_visible_to_with_sharing(&self, witness: &FileSharingWitness) -> bool {
        self.id == witness.file_id()
    }

    pub fn can_be_shared_by(&self, user: &User) -> bool {
        self.is_visible_to(user) && self.author_id == user.id
    }

    pub fn share_by(
        &self,
        user: &User,
        scope: FileSharingScope,
    ) -> Result<FileSharing, NonSharableFileError> {
        if !self.can_be_shared_by(user) {
            return Err(NonSharableFileError { _priv: () });
        }

        Ok(FileSharing::new(self.id, scope))
    }

    pub fn share_with_expiration_by(
        &self,
        user: &User,
        scope: FileSharingScope,
        expires_at: DateTime,
    ) -> Result<FileSharing, ShareWithExpirationError> {
        if !self.can_be_shared_by(user) {
            return Err(ShareWithExpirationError {
                kind: ShareWithExpirationErrorKind::NonSharableFile,
            });
        }

        FileSharing::with_expiration(self.id, scope, expires_at)
            .map_err(ShareWithExpirationError::from_sharing_expiration_error)
    }
}

#[cfg(test)]
mod tests {
    use super::FileSharingScope;

    use crate::model::date_time::DateTime;
    use crate::test::model as test_model;

    #[test]
    fn test_visibility_general_owner() {
        let user = test_model::new_general_user();
        let (file, _) = test_model::new_file(user.id.clone());
        assert!(file.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_other() {
        let user = test_model::new_general_user();
        let other = test_model::new_general_user();
        let (file, _) = test_model::new_file(other.id.clone());
        assert!(!file.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee_other() {
        let user = test_model::new_committee_user();
        let other = test_model::new_general_user();
        let (file, _) = test_model::new_file(other.id.clone());
        assert!(!file.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator_other() {
        let user = test_model::new_operator_user();
        let other = test_model::new_general_user();
        let (file, _) = test_model::new_file(other.id.clone());
        assert!(!file.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_admin_other() {
        let user = test_model::new_admin_user();
        let other = test_model::new_general_user();
        let (file, _) = test_model::new_file(other.id.clone());
        assert!(!file.is_visible_to(&user));
    }

    #[test]
    fn test_can_be_shared_by_general_owner() {
        let user = test_model::new_general_user();
        let (file, _) = test_model::new_file(user.id.clone());
        assert!(file.can_be_shared_by(&user));
    }

    #[test]
    fn test_can_be_shared_by_general_other() {
        let user = test_model::new_general_user();
        let other = test_model::new_general_user();
        let (file, _) = test_model::new_file(other.id.clone());
        assert!(!file.can_be_shared_by(&user));
    }

    #[test]
    fn test_can_be_shared_by_admin_other() {
        let user = test_model::new_admin_user();
        let other = test_model::new_general_user();
        let (file, _) = test_model::new_file(other.id.clone());
        assert!(!file.can_be_shared_by(&user));
    }

    #[test]
    fn test_share_by_general_owner() {
        let user = test_model::new_general_user();
        let (file, _) = test_model::new_file(user.id.clone());
        assert!(matches!(
            file.share_by(&user, FileSharingScope::Public),
            Ok(sharing)
            if sharing.file_id() == file.id
        ));
    }

    #[test]
    fn test_share_by_general_other() {
        use super::NonSharableFileError;

        let user = test_model::new_general_user();
        let other = test_model::new_general_user();
        let (file, _) = test_model::new_file(other.id.clone());
        assert!(matches!(
            file.share_by(&user, FileSharingScope::Public),
            Err(NonSharableFileError { .. })
        ));
    }

    #[test]
    fn test_share_by_admin_other() {
        use super::NonSharableFileError;

        let user = test_model::new_admin_user();
        let other = test_model::new_general_user();
        let (file, _) = test_model::new_file(other.id.clone());
        assert!(matches!(
            file.share_by(&user, FileSharingScope::Public),
            Err(NonSharableFileError { .. })
        ));
    }

    #[test]
    fn test_share_with_expiration_by_general_owner() {
        let user = test_model::new_general_user();
        let (file, _) = test_model::new_file(user.id.clone());
        let scope = FileSharingScope::Public;
        let expires_at = DateTime::from_utc(chrono::Utc::now() + chrono::Duration::days(1));
        assert!(matches!(
            file.share_with_expiration_by(&user, scope, expires_at),
            Ok(sharing)
            if sharing.file_id() == file.id
        ));
    }

    #[test]
    fn test_share_with_expiration_by_admin_other() {
        use super::ShareWithExpirationErrorKind;

        let user = test_model::new_admin_user();
        let other = test_model::new_general_user();
        let (file, _) = test_model::new_file(other.id.clone());
        let scope = FileSharingScope::Public;
        let expires_at = DateTime::from_utc(chrono::Utc::now() + chrono::Duration::days(1));
        assert!(matches!(
            file.share_with_expiration_by(&user, scope, expires_at),
            Err(err)
            if matches!(err.kind(), ShareWithExpirationErrorKind::NonSharableFile)
        ));
    }

    #[test]
    fn test_share_with_past_expiration_by_general_owner() {
        use super::ShareWithExpirationErrorKind;

        let user = test_model::new_general_user();
        let (file, _) = test_model::new_file(user.id.clone());
        let scope = FileSharingScope::Public;
        let expires_at = DateTime::from_utc(chrono::Utc::now() - chrono::Duration::days(1));
        assert!(matches!(
            file.share_with_expiration_by(&user, scope, expires_at),
            Err(err)
            if matches!(err.kind(), ShareWithExpirationErrorKind::InvalidExpirationDate)
        ));
    }
}
