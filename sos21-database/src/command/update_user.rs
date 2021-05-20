use crate::model::user::{UserAssignment, UserCategory, UserRole};

use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: String,
    pub first_name: String,
    pub kana_first_name: String,
    pub last_name: String,
    pub kana_last_name: String,
    pub phone_number: String,
    pub affiliation: Option<String>,
    pub role: UserRole,
    pub category: UserCategory,
    pub assignment: Option<UserAssignment>,
    pub assignment_owner_project_id: Option<Uuid>,
    pub assignment_subowner_project_id: Option<Uuid>,
    pub assignment_owner_pending_project_id: Option<Uuid>,
}

pub async fn update_user<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE users
  SET
    first_name = $2,
    kana_first_name = $3,
    last_name = $4,
    kana_last_name = $5,
    phone_number = $6,
    affiliation = $7,
    role = $8,
    category = $9,
    assignment = $10,
    assignment_owner_project_id = $11,
    assignment_subowner_project_id = $12,
    assignment_owner_pending_project_id = $13
  WHERE id = $1
"#,
        input.id,
        input.first_name,
        input.kana_first_name,
        input.last_name,
        input.kana_last_name,
        input.phone_number,
        input.affiliation,
        input.role as _,
        input.category as _,
        input.assignment as _,
        input.assignment_owner_project_id,
        input.assignment_subowner_project_id,
        input.assignment_owner_pending_project_id,
    )
    .execute(conn)
    .await
    .context("Failed to update on users")?;
    Ok(())
}
