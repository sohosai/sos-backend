use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn delete_user_invitation<'a, E>(conn: E, id: Uuid) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!("DELETE FROM user_invitations where id = $1", id)
        .execute(conn)
        .await
        .context("Failed to delete from user invitations")?;

    Ok(())
}
