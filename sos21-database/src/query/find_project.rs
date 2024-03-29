use crate::model::{
    project::{Project, ProjectAttributes, ProjectCategory, ProjectWithOwners},
    user::{User, UserAssignment, UserCategory, UserRole},
};

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_project<'a, E>(conn: E, id: Uuid) -> Result<Option<ProjectWithOwners>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let row = sqlx::query!(
        r#"
SELECT
        projects.id,
        projects.index,
        projects.created_at,
        projects.updated_at,
        projects.name,
        projects.kana_name,
        projects.group_name,
        projects.kana_group_name,
        projects.description,
        projects.category AS "category: ProjectCategory",
        projects.attributes AS "attributes: ProjectAttributes",
        owners.id AS owner_id,
        owners.created_at AS owner_created_at,
        owners.first_name AS owner_first_name,
        owners.kana_first_name AS owner_kana_first_name,
        owners.last_name AS owner_last_name,
        owners.kana_last_name AS owner_kana_last_name,
        owners.phone_number AS owner_phone_number,
        owners.email AS owner_email,
        owners.role AS "owner_role: UserRole",
        owners.category AS "owner_category: UserCategory",
        owners.assignment AS "owner_assignment: UserAssignment",
        owners.assignment_owner_project_id AS owner_assignment_owner_project_id,
        owners.assignment_subowner_project_id AS owner_assignment_subowner_project_id,
        owners.assignment_owner_pending_project_id AS owner_assignment_owner_pending_project_id,
        subowners.id AS subowner_id,
        subowners.created_at AS subowner_created_at,
        subowners.first_name AS subowner_first_name,
        subowners.kana_first_name AS subowner_kana_first_name,
        subowners.last_name AS subowner_last_name,
        subowners.kana_last_name AS subowner_kana_last_name,
        subowners.phone_number AS subowner_phone_number,
        subowners.email AS subowner_email,
        subowners.role AS "subowner_role: UserRole",
        subowners.category AS "subowner_category: UserCategory",
        subowners.assignment AS "subowner_assignment: UserAssignment",
        subowners.assignment_owner_project_id AS subowner_assignment_owner_project_id,
        subowners.assignment_subowner_project_id AS subowner_assignment_subowner_project_id,
        subowners.assignment_owner_pending_project_id AS subowner_assignment_owner_pending_project_id
FROM projects
INNER JOIN users AS owners ON (owners.assignment = 'project_owner' AND owners.assignment_owner_project_id = projects.id)
INNER JOIN users AS subowners ON (subowners.assignment = 'project_subowner' AND subowners.assignment_subowner_project_id = projects.id)
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
        index: row.index,
        created_at: row.created_at,
        updated_at: row.updated_at,
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
        email: row.owner_email,
        role: row.owner_role,
        category: row.owner_category,
        assignment: row.owner_assignment,
        assignment_owner_project_id: row.owner_assignment_owner_project_id,
        assignment_subowner_project_id: row.owner_assignment_subowner_project_id,
        assignment_owner_pending_project_id: row.owner_assignment_owner_pending_project_id,
    };
    let subowner = User {
        id: row.subowner_id,
        created_at: row.subowner_created_at,
        first_name: row.subowner_first_name,
        kana_first_name: row.subowner_kana_first_name,
        last_name: row.subowner_last_name,
        kana_last_name: row.subowner_kana_last_name,
        phone_number: row.subowner_phone_number,
        email: row.subowner_email,
        role: row.subowner_role,
        category: row.subowner_category,
        assignment: row.subowner_assignment,
        assignment_owner_project_id: row.subowner_assignment_owner_project_id,
        assignment_subowner_project_id: row.subowner_assignment_subowner_project_id,
        assignment_owner_pending_project_id: row.subowner_assignment_owner_pending_project_id,
    };

    Ok(Some(ProjectWithOwners {
        project,
        owner,
        subowner,
    }))
}
