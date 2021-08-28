use crate::model::registration_form_answer::RegistrationFormAnswer;

use anyhow::{Context, Result};

pub async fn insert_registration_form_answer<'a, E>(
    conn: E,
    answer: RegistrationFormAnswer,
) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let RegistrationFormAnswer {
        id,
        created_at,
        updated_at,
        author_id,
        registration_form_id,
        project_id,
        pending_project_id,
        items,
    } = answer;

    sqlx::query!(
        r#"
INSERT INTO registration_form_answers (
    id,
    created_at,
    updated_at,
    author_id,
    registration_form_id,
    project_id,
    pending_project_id,
    items
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8 )
"#,
        id,
        created_at,
        updated_at,
        author_id,
        registration_form_id,
        project_id,
        pending_project_id,
        items,
    )
    .execute(conn)
    .await
    .context("Failed to insert to registration form answers")?;

    Ok(())
}
