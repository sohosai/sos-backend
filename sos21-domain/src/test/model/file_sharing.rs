use crate::model::date_time::DateTime;
use crate::model::file::FileId;
use crate::model::file_sharing::{
    FileSharing, FileSharingContent, FileSharingId, FileSharingScope,
};

use uuid::Uuid;

pub fn new_expired_file_sharing(file_id: FileId, scope: FileSharingScope) -> FileSharing {
    let expires_at = DateTime::now();
    let created_at = DateTime::from_utc(expires_at.utc() - chrono::Duration::seconds(1));

    let sharing = FileSharingContent {
        id: FileSharingId::from_uuid(Uuid::new_v4()),
        created_at,
        file_id,
        is_revoked: false,
        expires_at: Some(expires_at),
        scope,
    };
    FileSharing::from_content(sharing)
}
