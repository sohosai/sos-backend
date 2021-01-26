use crate::model::project::{ProjectAttributes, ProjectCategory};

use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub owner_id: String,
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: ProjectAttributes,
}

pub async fn update_project<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query(
        r#"
UPDATE projects
  SET
    owner_id = $2,
    name = $3,
    kana_name = $4,
    group_name = $5,
    kana_group_name = $6,
    description = $7,
    category = $8,
    attributes = $9
  WHERE id = $1
"#,
    )
    .bind(input.id)
    .bind(input.owner_id)
    .bind(input.name)
    .bind(input.kana_name)
    .bind(input.group_name)
    .bind(input.kana_group_name)
    .bind(input.description)
    .bind(input.category)
    .bind(input.attributes)
    .execute(conn)
    .await
    .context("Failed to update on projects")?;
    Ok(())
}
