use crate::model::pending_project::{PendingProject, PendingProjectWithOwner};
use crate::model::project::{ProjectAttributes, ProjectCategory};
use crate::model::user::{User, UserAssignment, UserCategory, UserRole};

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_pending_project<'a, E>(
    conn: E,
    id: Uuid,
) -> Result<Option<PendingProjectWithOwner>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let row = sqlx::query!(
        r#"
SELECT
        pending_projects.id,
        pending_projects.created_at,
        pending_projects.updated_at,
        pending_projects.name,
        pending_projects.kana_name,
        pending_projects.group_name,
        pending_projects.kana_group_name,
        pending_projects.description,
        pending_projects.category AS "category: ProjectCategory",
        pending_projects.attributes AS "attributes: ProjectAttributes",
        pending_projects.exceptional_complete_deadline,
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
        owners.assignment_owner_pending_project_id AS owner_assignment_owner_pending_project_id
FROM pending_projects
INNER JOIN users AS owners ON (
    owners.assignment = 'pending_project_owner'
    AND owners.assignment_owner_pending_project_id = pending_projects.id
)
WHERE pending_projects.id = $1
"#,
        id
    )
    .fetch_optional(conn)
    .await
    .context("Failed to select from pending projects")?;

    let row = match row {
        Some(row) => row,
        None => return Ok(None),
    };
    let pending_project = PendingProject {
        id: row.id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        name: row.name,
        kana_name: row.kana_name,
        group_name: row.group_name,
        kana_group_name: row.kana_group_name,
        description: row.description,
        category: row.category,
        attributes: row.attributes,
        exceptional_complete_deadline: row.exceptional_complete_deadline,
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

    Ok(Some(PendingProjectWithOwner {
        pending_project,
        owner,
    }))
}
