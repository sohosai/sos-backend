use crate::model::form_answer::FormAnswer;

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};
use uuid::Uuid;

pub fn list_form_answers_by_form<'a, E>(conn: E, form_id: Uuid) -> BoxStream<'a, Result<FormAnswer>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    sqlx::query_as!(
        FormAnswer,
        "SELECT * FROM form_answers WHERE form_id = $1",
        form_id
    )
    .fetch(conn)
    .map(|result| result.context("Failed to select from form_answers"))
    .boxed()
}
