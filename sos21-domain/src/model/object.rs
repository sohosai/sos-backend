use crate::model::file::File;
use crate::model::file_sharing::FileSharingWitness;
use crate::model::user::User;

use uuid::Uuid;

pub mod data;
pub use data::ObjectData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(Uuid);

impl ObjectId {
    pub fn from_uuid(uuid: Uuid) -> ObjectId {
        ObjectId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug)]
pub struct Object {
    pub id: ObjectId,
    pub data: ObjectData,
}

impl Object {
    pub fn is_visible_to(&self, _user: &User) -> bool {
        false
    }

    pub fn is_visible_to_with_file(&self, user: &User, file: &File) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.id == file.object_id && file.is_visible_to(user)
    }

    pub fn is_visible_to_with_sharing(&self, file: &File, witness: &FileSharingWitness) -> bool {
        self.id == file.object_id && file.is_visible_to_with_sharing(witness)
    }
}
