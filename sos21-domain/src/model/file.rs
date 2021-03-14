use crate::model::date_time::DateTime;
use crate::model::object::ObjectId;
use crate::model::user::{User, UserId};

use uuid::Uuid;

pub mod name;
pub use name::FileName;
pub mod type_;
pub use type_::FileType;
pub mod digest;
pub use digest::FileBlake3Digest;

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
    pub size: u64,
}

impl File {
    pub fn is_visible_to(&self, user: &User) -> bool {
        self.author_id == user.id
    }
}

#[cfg(test)]
mod tests {
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
}
