use crate::model::{
    project::{Project, ProjectAttributes, ProjectCategory},
    user::{User, UserRole},
};

use anyhow::{Context, Result};
use uuid::Uuid;

pub struct ProjectWithOwner {
    pub project: Project,
    pub owner: User,
}

pub async fn find_project<'a, E>(conn: E, id: Uuid) -> Result<Option<ProjectWithOwner>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let row = sqlx::query!(
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
        category as "category: ProjectCategory",
        attributes as "attributes: ProjectAttributes",
        users.id as user_id,
        users.created_at as user_created_at,
        first_name,
        kana_first_name,
        last_name,
        kana_last_name,
        phone_number,
        affiliation,
        email,
        role as "role: UserRole"
FROM projects
INNER JOIN users ON (projects.owner_id = users.id)
WHERE projects.id = $1
"#,
        id
    )
    .fetch_optional(conn)
    .await
    .context("Failed to select from projects")?;
    let row = match row {
        Some(row) => row,
        None => return Ok(None),
    };
    let project = Project {
        id: row.id,
        created_at: row.created_at,
        owner_id: row.owner_id,
        name: row.name,
        kana_name: row.kana_name,
        group_name: row.group_name,
        kana_group_name: row.kana_group_name,
        description: row.description,
        category: row.category,
        attributes: row.attributes,
    };
    let owner = User {
        id: row.user_id,
        created_at: row.user_created_at,
        first_name: row.first_name,
        kana_first_name: row.kana_first_name,
        last_name: row.last_name,
        kana_last_name: row.kana_last_name,
        phone_number: row.phone_number,
        affiliation: row.affiliation,
        email: row.email,
        role: row.role,
    };
    Ok(Some(ProjectWithOwner { owner, project }))
}
