use crate::model::{
    file::File,
    file_sharing::{FileSharing, FileSharingScope},
};

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};

#[derive(Debug, Clone)]
pub struct FileWithSharing {
    pub file: File,
    pub sharing: FileSharing,
}

pub fn list_file_sharings_by_user<'a, E>(
    conn: E,
    user_id: String,
) -> BoxStream<'a, Result<FileWithSharing>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    sqlx::query!(
        r#"
SELECT
    file_sharings.id,
    file_sharings.created_at,
    file_sharings.file_id,
    file_sharings.is_revoked,
    file_sharings.expires_at,
    file_sharings.scope AS "scope: FileSharingScope",
    file_sharings.project_id,
    file_sharings.form_answer_project_id,
    file_sharings.form_answer_form_id,
    file_sharings.registration_form_answer_project_id,
    file_sharings.registration_form_answer_pending_project_id,
    file_sharings.registration_form_answer_registration_form_id,
    files.created_at AS file_created_at,
    files.author_id AS file_author_id,
    files.object_id AS file_object_id,
    files.blake3_digest AS file_blake3_digest,
    files.name AS file_name,
    files.type_ AS file_type,
    files.size AS file_size
FROM file_sharings
INNER JOIN files ON (file_sharings.file_id = files.id)
WHERE files.author_id = $1
"#,
        user_id
    )
    .fetch(conn)
    .map(|row| {
        let row = row.context("Failed to select from file sharings")?;

        let sharing = FileSharing {
            id: row.id,
            created_at: row.created_at,
            file_id: row.file_id,
            is_revoked: row.is_revoked,
            expires_at: row.expires_at,
            scope: row.scope,
            project_id: row.project_id,
            form_answer_project_id: row.form_answer_project_id,
            form_answer_form_id: row.form_answer_form_id,
            registration_form_answer_project_id: row.registration_form_answer_project_id,
            registration_form_answer_pending_project_id: row
                .registration_form_answer_pending_project_id,
            registration_form_answer_registration_form_id: row
                .registration_form_answer_registration_form_id,
        };

        let file = File {
            id: row.file_id,
            created_at: row.file_created_at,
            author_id: row.file_author_id,
            object_id: row.file_object_id,
            blake3_digest: row.file_blake3_digest,
            name: row.file_name,
            type_: row.file_type,
            size: row.file_size,
        };

        Ok(FileWithSharing { file, sharing })
    })
    .boxed()
}
