use crate::model::form_answer::FormAnswer;

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_form_answer_by_form_and_project<'a, E>(
    conn: E,
    form_id: Uuid,
    project_id: Uuid,
) -> Result<Option<FormAnswer>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        FormAnswer,
        "SELECT * FROM form_answers WHERE form_id = $1 AND project_id = $2",
        form_id,
        project_id
    )
    .fetch_optional(conn)
    .await
    .context("Failed to select from form answers")
}
