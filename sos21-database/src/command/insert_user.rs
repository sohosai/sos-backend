use crate::model::user::User;

use anyhow::{Context, Result};

pub async fn insert_user<'a, E>(conn: E, user: User) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let User {
        id,
        created_at,
        first_name,
        kana_first_name,
        last_name,
        kana_last_name,
        email,
        phone_number,
        affiliation,
        role,
        category,
        assignment,
        assignment_owner_project_id,
        assignment_subowner_project_id,
        assignment_owner_pending_project_id,
    } = user;

    sqlx::query!(
        r#"
INSERT INTO users (
    id,
    created_at,
    first_name,
    kana_first_name,
    last_name,
    kana_last_name,
    email,
    phone_number,
    affiliation,
    role,
    category,
    assignment,
    assignment_owner_project_id,
    assignment_subowner_project_id,
    assignment_owner_pending_project_id
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15 )
"#,
        id,
        created_at,
        first_name,
        kana_first_name,
        last_name,
        kana_last_name,
        email,
        phone_number,
        affiliation,
        role as _,
        category as _,
        assignment as _,
        assignment_owner_project_id,
        assignment_subowner_project_id,
        assignment_owner_pending_project_id
    )
    .execute(conn)
    .await
    .context("Failed to insert to users")?;

    Ok(())
}
