use crate::model::form_answer::FormAnswer;

use anyhow::{Context, Result};

pub async fn insert_form_answer<'a, E>(conn: E, answer: FormAnswer) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let FormAnswer {
        id,
        created_at,
        author_id,
        form_id,
        project_id,
        items,
    } = answer;

    sqlx::query!(
        r#"
INSERT INTO form_answers (
    id,
    created_at,
    author_id,
    form_id,
    project_id,
    items
) VALUES ( $1, $2, $3, $4, $5, $6 )
"#,
        id,
        created_at,
        author_id,
        form_id,
        project_id,
        items,
    )
    .execute(conn)
    .await
    .context("Failed to insert to form answers")?;

    Ok(())
}
