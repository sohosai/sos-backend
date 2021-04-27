use crate::model::project::{ProjectAttributes, ProjectCategory};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: ProjectAttributes,
}

pub async fn update_pending_project<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let Input {
        id,
        created_at,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
    } = input;

    sqlx::query!(
        r#"
UPDATE pending_projects
  SET
    created_at = $2,
    name = $3,
    kana_name = $4,
    group_name = $5,
    kana_group_name = $6,
    description = $7,
    category = $8,
    attributes = $9
  WHERE id = $1
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
    .context("Failed to update on pending projects")?;

    Ok(())
}
