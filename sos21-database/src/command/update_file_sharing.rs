use crate::model::file_sharing::FileSharingScope;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub file_id: Uuid,
    pub is_revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: FileSharingScope,
    pub project_id: Option<Uuid>,
    pub form_answer_project_id: Option<Uuid>,
    pub form_answer_form_id: Option<Uuid>,
}

pub async fn update_file_sharing<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE file_sharings
  SET
    file_id = $2,
    is_revoked = $3,
    expires_at = $4,
    scope = $5,
    project_id = $6,
    form_answer_project_id = $7,
    form_answer_form_id = $8
  WHERE id = $1
"#,
        input.id,
        input.file_id,
        input.is_revoked,
        input.expires_at,
        input.scope as _,
        input.project_id,
        input.form_answer_project_id,
        input.form_answer_form_id,
    )
    .execute(conn)
    .await
    .context("Failed to update on file sharings")?;

    Ok(())
}
