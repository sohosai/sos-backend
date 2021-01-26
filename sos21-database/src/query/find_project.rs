use crate::model::{project::Project, user::User};

use anyhow::{Context, Result};
use sqlx::{FromRow, Row};
use uuid::Uuid;

pub struct ProjectWithOwner {
    pub project: Project,
    pub owner: User,
}

pub async fn find_project<'a, E>(conn: E, id: Uuid) -> Result<Option<ProjectWithOwner>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    // TODO: Concise row interpretation in presence of JOIN
    let row = sqlx::query(
        r#"
SELECT
        projects.id,
        projects.created_at,
        owner_id,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
        users.id as user_id,
        users.created_at as user_created_at,
        first_name,
        kana_first_name,
        last_name,
        kana_last_name,
        email,
        role
FROM projects
INNER JOIN users ON (projects.owner_id = users.id)
WHERE projects.id = $1
"#,
    )
    .bind(id)
    .fetch_optional(conn)
    .await
    .context("Failed to select from projects")?;
    let row = match row {
        Some(row) => row,
        None => return Ok(None),
    };
    let project = Project::from_row(&row)?;
    let owner = User {
        id: row.try_get("user_id")?,
        created_at: row.try_get("user_created_at")?,
        first_name: row.try_get("first_name")?,
        kana_first_name: row.try_get("kana_first_name")?,
        last_name: row.try_get("last_name")?,
        kana_last_name: row.try_get("kana_last_name")?,
        email: row.try_get("email")?,
        role: row.try_get("role")?,
    };
    Ok(Some(ProjectWithOwner { owner, project }))
}
