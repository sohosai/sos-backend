use crate::model::form_answer::FormAnswer;

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_form_answer<'a, E>(conn: E, id: Uuid) -> Result<Option<FormAnswer>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(FormAnswer, "SELECT * FROM form_answers WHERE id = $1", id)
        .fetch_optional(conn)
        .await
        .context("Failed to select from form answers")
}
