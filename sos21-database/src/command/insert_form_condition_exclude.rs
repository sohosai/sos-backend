use crate::model::form::FormConditionExclude;

use anyhow::{Context, Result};

pub async fn insert_form_condition_exclude<'a, E>(
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
INSERT INTO form_condition_excludes (
        project_id,
        form_id
) VALUES ( $1, $2 )
"#,
        project_id,
        form_id
    )
    .execute(conn)
    .await
    .context("Failed to insert to form condition excludes")?;

    Ok(())
}
