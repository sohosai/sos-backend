use crate::model::form::FormConditionInclude;

use anyhow::{Context, Result};

pub async fn insert_form_condition_include<'a, E>(
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
INSERT INTO form_condition_includes (
        project_id,
        form_id
) VALUES ( $1, $2 )
"#,
        project_id,
        form_id
    )
    .execute(conn)
    .await
    .context("Failed to insert to form condition includes")?;

    Ok(())
}
