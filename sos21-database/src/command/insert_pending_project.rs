use crate::model::pending_project::PendingProject;

use anyhow::{Context, Result};

pub async fn insert_pending_project<'a, E>(conn: E, pending_project: PendingProject) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let PendingProject {
        id,
        created_at,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
    } = pending_project;

    sqlx::query!(
        r#"
INSERT INTO pending_projects (
    id,
    created_at,
    name,
    kana_name,
    group_name,
    kana_group_name,
    description,
    category,
    attributes
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9 )
"#,
        id,
        created_at,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category as _,
        attributes as _,
    )
    .execute(conn)
    .await
    .context("Failed to insert to pending projects")?;

    Ok(())
}
