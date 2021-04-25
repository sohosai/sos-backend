use std::convert::TryInto;

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn count_registration_forms_by_pending_project<'a, E>(
    conn: E,
    pending_project_id: Uuid,
) -> Result<u64>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let count = sqlx::query_scalar!(
        r#"
SELECT count(registration_forms.id)
FROM registration_forms
WHERE (
    SELECT
        bool_or((
            registration_form_project_query_conjunctions.category = pending_projects.category IS NOT FALSE
            AND registration_form_project_query_conjunctions.attributes | pending_projects.attributes = pending_projects.attributes
        ))
    FROM registration_form_project_query_conjunctions, pending_projects
    WHERE registration_form_project_query_conjunctions.registration_form_id = registration_forms.id AND pending_projects.id = $1
)
"#,
        pending_project_id
    )
    .fetch_one(conn)
    .await
    .context("Failed to count registration forms")?;

    let count = count.unwrap_or(0).try_into()?;
    Ok(count)
}
