use crate::model::registration_form_answer::RegistrationFormAnswer;

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};
use uuid::Uuid;

pub fn list_registration_form_answers_by_pending_project<'a, 'b, E>(
    conn: E,
    pending_project_id: Uuid,
) -> BoxStream<'b, Result<RegistrationFormAnswer>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'b,
    'a: 'b,
{
    sqlx::query_as!(
        RegistrationFormAnswer,
        "SELECT * FROM registration_form_answers WHERE pending_project_id = $1",
        pending_project_id
    )
    .fetch(conn)
    .map(|result| result.context("Failed to select from registration form answers"))
    .boxed()
}
