use crate::model::{
    project::{Project, ProjectAttributes, ProjectCategory},
    user::{User, UserRole},
};

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};

pub struct ProjectWithOwner {
    pub project: Project,
    pub owner: User,
}

pub fn list_projects<'a, E>(conn: E) -> BoxStream<'a, Result<ProjectWithOwner>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    // TODO: Remove tedeous null forcings
    sqlx::query!(
        r#"
SELECT
        projects.id as "id!",
        projects.created_at as "created_at!",
        display_id as "display_id!",
        owner_id as "owner_id!",
        name as "name!",
        kana_name as "kana_name!",
        group_name as "group_name!",
        kana_group_name as "kana_group_name!",
        description as "description!",
        category as "category!: ProjectCategory",
        attributes as "attributes!: ProjectAttributes",
        users.id as "user_id!",
        users.created_at as "user_created_at!",
        first_name as "first_name!",
        kana_first_name as "kana_first_name!",
        last_name as "last_name!",
        kana_last_name as "kana_last_name!",
        phone_number as "phone_number!",
        affiliation as "affiliation!",
        email as "email!",
        role as "role!: UserRole"
FROM projects
INNER JOIN users ON (projects.owner_id = users.id)
"#,
    )
    .fetch(conn)
    .map(|row| {
        let row = row.context("Failed to select from projects")?;
        let project = Project {
            id: row.id,
            created_at: row.created_at,
            display_id: row.display_id,
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
        Ok(ProjectWithOwner { owner, project })
    })
    .boxed()
}
