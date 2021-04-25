use crate::model::registration_form_answer::RegistrationFormAnswer;

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_registration_form_answer<'a, E>(
    conn: E,
    id: Uuid,
) -> Result<Option<RegistrationFormAnswer>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        RegistrationFormAnswer,
        "SELECT * FROM registration_form_answers WHERE id = $1",
        id
    )
    .fetch_optional(conn)
    .await
    .context("Failed to select from registration form answers")
}
