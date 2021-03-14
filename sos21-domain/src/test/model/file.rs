use crate::model::{
    date_time::DateTime,
    file::{File, FileBlake3Digest, FileId, FileType},
    object::{Object, ObjectId},
    user::UserId,
};
use crate::test::model as test_model;
use uuid::Uuid;

pub fn new_file_id() -> FileId {
    FileId::from_uuid(Uuid::new_v4())
}

pub fn mock_file_type() -> FileType {
    FileType::from_mime(mime::IMAGE_PNG)
}

pub fn new_file_with_object(
    author_id: UserId,
    object_id: ObjectId,
    object_blake3: [u8; 32],
    object_size: u64,
) -> File {
    File {
        id: new_file_id(),
        created_at: DateTime::now(),
        author_id,
        object_id,
        blake3_digest: FileBlake3Digest::from_array(object_blake3),
        name: None,
        type_: mock_file_type(),
        size: object_size,
    }
}

pub fn new_file(author_id: UserId) -> (File, Object) {
    let (object, digest, size) = test_model::new_object();
    (
        new_file_with_object(author_id, object.id, digest, size),
        object,
    )
}
