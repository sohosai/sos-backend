use crate::model::pending_project::PendingProject;

use anyhow::{Context, Result};

pub async fn insert_pending_project<'a, E>(conn: E, pending_project: PendingProject) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let PendingProject {
        id,
        created_at,
        updated_at,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
        exceptional_complete_deadline,
    } = pending_project;

    sqlx::query!(
        r#"
INSERT INTO pending_projects (
    id,
    created_at,
    updated_at,
    name,
    kana_name,
    group_name,
    kana_group_name,
    description,
    category,
    attributes,
    exceptional_complete_deadline
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10 , $11)
"#,
        id,
        created_at,
        updated_at,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category as _,
        attributes as _,
        exceptional_complete_deadline
    )
    .execute(conn)
    .await
    .context("Failed to insert to pending projects")?;

    Ok(())
}
