use crate::model::form::FormConditionInclude;

use anyhow::{Context, Result};

pub async fn delete_form_condition_include<'a, E>(
    conn: E,
    form_condition_include: FormConditionInclude,
) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let FormConditionInclude {
        project_id,
        form_id,
    } = form_condition_include;

    sqlx::query!(
        r#"
DELETE FROM form_condition_includes
WHERE project_id = $1 AND form_id = $2
"#,
        project_id,
        form_id
    )
    .execute(conn)
    .await
    .context("Failed to delete form condition includes")?;

    Ok(())
}
