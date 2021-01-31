use crate::model::project::Project;

use anyhow::{Context, Result};

pub async fn insert_project<'a, E>(conn: E, project: Project) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let Project {
        id,
        created_at,
        display_id,
        owner_id,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
    } = project;

    sqlx::query!(
        r#"
INSERT INTO projects (
    id,
    created_at,
    display_id,
    owner_id,
    name,
    kana_name,
    group_name,
    kana_group_name,
    description,
    category,
    attributes
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11 )
"#,
        id,
        created_at,
        display_id,
        owner_id,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category as _,
        attributes as _
    )
    .execute(conn)
    .await
    .context("Failed to insert to projects")?;

    Ok(())
}
