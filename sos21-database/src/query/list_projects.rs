use crate::model::{project::Project, user::User};

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};
use sqlx::{FromRow, Row};

pub struct ProjectWithOwner {
    pub project: Project,
    pub owner: User,
}

pub fn list_projects<'a, E>(conn: E) -> BoxStream<'a, Result<ProjectWithOwner>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    // TODO: Concise row interpretation in presence of JOIN
    sqlx::query(
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
"#,
    )
    .fetch(conn)
    .map(|row| {
        let row = row.context("Failed to select from projects")?;
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
        Ok(ProjectWithOwner { owner, project })
    })
    .boxed()
}
