use crate::model::project::Project;

use anyhow::{Context, Result};

pub async fn insert_project<'a, E>(conn: E, project: Project) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let Project {
        id,
        created_at,
        owner_id,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
    } = project;

    sqlx::query(
        r#"
INSERT INTO projects (
    id,
    created_at,
    owner_id,
    name,
    kana_name,
    group_name,
    kana_group_name,
    description,
    category,
    attributes
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10 )
"#,
    )
    .bind(id)
    .bind(created_at)
    .bind(owner_id)
    .bind(name)
    .bind(kana_name)
    .bind(group_name)
    .bind(kana_group_name)
    .bind(description)
    .bind(category)
    .bind(attributes)
    .execute(conn)
    .await
    .context("Failed to insert to projects")?;

    Ok(())
}
