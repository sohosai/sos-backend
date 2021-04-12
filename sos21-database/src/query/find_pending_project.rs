use crate::model::pending_project::{PendingProject, PendingProjectWithAuthor};
use crate::model::project::{ProjectAttributes, ProjectCategory};
use crate::model::user::{User, UserCategory, UserRole};

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_pending_project<'a, E>(
    conn: E,
    id: Uuid,
) -> Result<Option<PendingProjectWithAuthor>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let row = sqlx::query!(
        r#"
SELECT
        pending_projects.id,
        pending_projects.created_at,
        pending_projects.author_id,
        pending_projects.name,
        pending_projects.kana_name,
        pending_projects.group_name,
        pending_projects.kana_group_name,
        pending_projects.description,
        pending_projects.category AS "category: ProjectCategory",
        pending_projects.attributes AS "attributes: ProjectAttributes",
        authors.created_at AS author_created_at,
        authors.first_name AS author_first_name,
        authors.kana_first_name AS author_kana_first_name,
        authors.last_name AS author_last_name,
        authors.kana_last_name AS author_kana_last_name,
        authors.phone_number AS author_phone_number,
        authors.affiliation AS author_affiliation,
        authors.email AS author_email,
        authors.role AS "author_role: UserRole",
        authors.category AS "author_category: UserCategory"
FROM pending_projects
INNER JOIN users AS authors ON (pending_projects.author_id = authors.id)
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
        author_id: row.author_id.clone(),
        name: row.name,
        kana_name: row.kana_name,
        group_name: row.group_name,
        kana_group_name: row.kana_group_name,
        description: row.description,
        category: row.category,
        attributes: row.attributes,
    };
    let author = User {
        id: row.author_id,
        created_at: row.author_created_at,
        first_name: row.author_first_name,
        kana_first_name: row.author_kana_first_name,
        last_name: row.author_last_name,
        kana_last_name: row.author_kana_last_name,
        phone_number: row.author_phone_number,
        affiliation: row.author_affiliation,
        email: row.author_email,
        role: row.author_role,
        category: row.author_category,
    };

    Ok(Some(PendingProjectWithAuthor {
        pending_project,
        author,
    }))
}
