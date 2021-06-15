use crate::model::file_sharing::FileSharing;

use anyhow::{Context, Result};

pub async fn insert_file_sharing<'a, E>(conn: E, sharing: FileSharing) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let FileSharing {
        id,
        created_at,
        file_id,
        is_revoked,
        expires_at,
        scope,
        project_id,
        project_query,
        form_answer_project_id,
        form_answer_form_id,
        registration_form_answer_project_id,
        registration_form_answer_pending_project_id,
        registration_form_answer_registration_form_id,
    } = sharing;

    sqlx::query!(
        r#"
INSERT INTO file_sharings (
    id,
    created_at,
    file_id,
    is_revoked,
    expires_at,
    scope,
    project_id,
    project_query,
    form_answer_project_id,
    form_answer_form_id,
    registration_form_answer_project_id,
    registration_form_answer_pending_project_id,
    registration_form_answer_registration_form_id
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13 )
"#,
        id,
        created_at,
        file_id,
        is_revoked,
        expires_at,
        scope as _,
        project_id,
        project_query,
        form_answer_project_id,
        form_answer_form_id,
        registration_form_answer_project_id,
        registration_form_answer_pending_project_id,
        registration_form_answer_registration_form_id,
    )
    .execute(conn)
    .await
    .context("Failed to insert to file sharings")?;

    Ok(())
}
