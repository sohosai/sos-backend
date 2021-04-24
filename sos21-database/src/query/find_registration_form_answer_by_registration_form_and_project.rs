use crate::model::registration_form_answer::RegistrationFormAnswer;

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_registration_form_answer_by_registration_form_and_project<'a, E>(
    conn: E,
    registration_form_id: Uuid,
    project_id: Uuid,
) -> Result<Option<RegistrationFormAnswer>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        RegistrationFormAnswer,
        "SELECT * FROM registration_form_answers WHERE registration_form_id = $1 AND project_id = $2",
        registration_form_id,
        project_id
    )
    .fetch_optional(conn)
    .await
    .context("Failed to select from registration form answers")
}
