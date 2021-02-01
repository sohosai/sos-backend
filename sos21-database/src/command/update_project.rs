use crate::model::project::{ProjectAttributes, ProjectCategory};

use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub owner_id: String,
    pub display_id: String,
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
    sqlx::query!(
        r#"
UPDATE projects
  SET
    owner_id = $2,
    display_id = $3,
    name = $4,
    kana_name = $5,
    group_name = $6,
    kana_group_name = $7,
    description = $8,
    category = $9,
    attributes = $10
  WHERE id = $1
"#,
        input.id,
        input.owner_id,
        input.display_id,
        input.name,
        input.kana_name,
        input.group_name,
        input.kana_group_name,
        input.description,
        input.category as _,
        input.attributes as _
    )
    .execute(conn)
    .await
    .context("Failed to update on projects")?;
    Ok(())
}
