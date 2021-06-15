use crate::model::file_sharing::{FileSharing, FileSharingScope};

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};
use uuid::Uuid;

pub fn list_file_sharings_by_pending_project<'a, E>(
    conn: E,
    pending_project_id: Uuid,
) -> BoxStream<'a, Result<FileSharing>>
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
    file_sharings.project_query,
    file_sharings.form_answer_project_id,
    file_sharings.form_answer_form_id,
    file_sharings.registration_form_answer_project_id,
    file_sharings.registration_form_answer_pending_project_id,
    file_sharings.registration_form_answer_registration_form_id
FROM file_sharings
WHERE file_sharings.registration_form_answer_pending_project_id = $1
"#,
        pending_project_id
    )
    .fetch(conn)
    .map(|row| {
        let row = row.context("Failed to select from file sharings")?;

        Ok(FileSharing {
            id: row.id,
            created_at: row.created_at,
            file_id: row.file_id,
            is_revoked: row.is_revoked,
            expires_at: row.expires_at,
            scope: row.scope,
            project_id: row.project_id,
            project_query: row.project_query,
            form_answer_project_id: row.form_answer_project_id,
            form_answer_form_id: row.form_answer_form_id,
            registration_form_answer_project_id: row.registration_form_answer_project_id,
            registration_form_answer_pending_project_id: row
                .registration_form_answer_pending_project_id,
            registration_form_answer_registration_form_id: row
                .registration_form_answer_registration_form_id,
        })
    })
    .boxed()
}
