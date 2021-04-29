use crate::model::user_invitation::UserInvitation;

use anyhow::{Context, Result};

pub async fn insert_user_invitation<'a, E>(conn: E, invitation: UserInvitation) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let UserInvitation {
        id,
        created_at,
        author_id,
        email,
        role,
    } = invitation;

    sqlx::query!(
        r#"
INSERT INTO user_invitations (
    id,
    created_at,
    author_id,
    email,
    role
) VALUES ( $1, $2, $3, $4, $5 )
"#,
        id,
        created_at,
        author_id,
        email,
        role as _
    )
    .execute(conn)
    .await
    .context("Failed to insert to user invitations")?;

    Ok(())
}
