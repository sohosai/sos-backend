use crate::model::user_invitation::UserInvitationRole;

use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub email: String,
    pub role: UserInvitationRole,
}

pub async fn update_user_invitation<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE user_invitations
  SET
    email = $2,
    role = $3
  WHERE id = $1
"#,
        input.id,
        input.email,
        input.role as _,
    )
    .execute(conn)
    .await
    .context("Failed to update on user invitations")?;

    Ok(())
}
