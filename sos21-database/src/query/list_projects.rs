use crate::model::{
    project::{Project, ProjectAttributes, ProjectCategory, ProjectWithOwners},
    user::{User, UserCategory, UserRole},
};

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};

pub fn list_projects<'a, E>(conn: E) -> BoxStream<'a, Result<ProjectWithOwners>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    // TODO: Remove tedeous null forcings
    sqlx::query!(
        r#"
SELECT
        projects.id AS "id!",
        projects.index AS "index!",
        projects.created_at AS "created_at!",
        projects.owner_id AS "owner_id!",
        projects.subowner_id AS "subowner_id!",
        projects.name AS "name!",
        projects.kana_name AS "kana_name!",
        projects.group_name AS "group_name!",
        projects.kana_group_name AS "kana_group_name!",
        projects.description AS "description!",
        projects.category AS "category!: ProjectCategory",
        projects.attributes AS "attributes!: ProjectAttributes",
        owners.created_at AS "owner_created_at!",
        owners.first_name AS "owner_first_name!",
        owners.kana_first_name AS "owner_kana_first_name!",
        owners.last_name AS "owner_last_name!",
        owners.kana_last_name AS "owner_kana_last_name!",
        owners.phone_number AS "owner_phone_number!",
        owners.affiliation AS "owner_affiliation!",
        owners.email AS "owner_email!",
        owners.role AS "owner_role!: UserRole",
        owners.category AS "owner_category!: UserCategory",
        subowners.created_at AS "subowner_created_at!",
        subowners.first_name AS "subowner_first_name!",
        subowners.kana_first_name AS "subowner_kana_first_name!",
        subowners.last_name AS "subowner_last_name!",
        subowners.kana_last_name AS "subowner_kana_last_name!",
        subowners.phone_number AS "subowner_phone_number!",
        subowners.affiliation AS "subowner_affiliation!",
        subowners.email AS "subowner_email!",
        subowners.role AS "subowner_role!: UserRole",
        subowners.category AS "subowner_category!: UserCategory"
FROM projects
INNER JOIN users AS owners ON (projects.owner_id = owners.id)
INNER JOIN users AS subowners ON (projects.subowner_id = subowners.id)
"#,
    )
    .fetch(conn)
    .map(|row| {
        let row = row.context("Failed to select from projects")?;

        let project = Project {
            id: row.id,
            index: row.index,
            created_at: row.created_at,
            owner_id: row.owner_id.clone(),
            subowner_id: row.subowner_id.clone(),
            name: row.name,
            kana_name: row.kana_name,
            group_name: row.group_name,
            kana_group_name: row.kana_group_name,
            description: row.description,
            category: row.category,
            attributes: row.attributes,
        };
        let owner = User {
            id: row.owner_id,
            created_at: row.owner_created_at,
            first_name: row.owner_first_name,
            kana_first_name: row.owner_kana_first_name,
            last_name: row.owner_last_name,
            kana_last_name: row.owner_kana_last_name,
            phone_number: row.owner_phone_number,
            affiliation: row.owner_affiliation,
            email: row.owner_email,
            role: row.owner_role,
            category: row.owner_category,
        };
        let subowner = User {
            id: row.subowner_id,
            created_at: row.subowner_created_at,
            first_name: row.subowner_first_name,
            kana_first_name: row.subowner_kana_first_name,
            last_name: row.subowner_last_name,
            kana_last_name: row.subowner_kana_last_name,
            phone_number: row.subowner_phone_number,
            affiliation: row.subowner_affiliation,
            email: row.subowner_email,
            role: row.subowner_role,
            category: row.subowner_category,
        };

        Ok(ProjectWithOwners {
            owner,
            subowner,
            project,
        })
    })
    .boxed()
}
