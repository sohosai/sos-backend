use std::convert::TryInto;

use anyhow::{Context, Result};

pub async fn is_healthy<'a, E>(conn: E) -> Result<bool>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let table_names: &[&str] = [
        "users",
        "projects",
        "forms",
        "form_condition_includes",
        "form_condition_excludes",
        "form_project_query_conjunctions",
        "form_answers",
        "file_sharings",
        "files",
        "file_distributions",
        "file_distribution_files",
        "pending_projects",
    ]
    .as_ref();

    let table_names_len: i64 = table_names.len().try_into()?;

    let has_grants = sqlx::query_scalar!(
        r#"
WITH grants AS (
    SELECT
        array_agg(privilege_type::text) AS privilege_types,
        table_name::text
    FROM information_schema.role_table_grants
    WHERE grantee = current_user AND table_name::text = ANY ($1)
    GROUP BY table_name
)
SELECT
    (bool_and(grants.privilege_types @> ARRAY['DELETE', 'UPDATE', 'SELECT', 'INSERT'])
        AND count(grants.table_name) = $2
    ) AS "has_grants!"
FROM grants
"#,
        table_names as _,
        table_names_len
    )
    .fetch_one(conn)
    .await
    .context("Failed to select from role_table_grants")?;

    Ok(has_grants)
}
