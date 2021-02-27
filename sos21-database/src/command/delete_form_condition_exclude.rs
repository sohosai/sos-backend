use crate::model::form::FormConditionExclude;

use anyhow::{Context, Result};

pub async fn delete_form_condition_exclude<'a, E>(
    conn: E,
    form_condition_exclude: FormConditionExclude,
) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let FormConditionExclude {
        project_id,
        form_id,
    } = form_condition_exclude;

    sqlx::query!(
        r#"
DELETE FROM form_condition_excludes
WHERE project_id = $1 AND form_id = $2
"#,
        project_id,
        form_id
    )
    .execute(conn)
    .await
    .context("Failed to delete form condition excludes")?;

    Ok(())
}
