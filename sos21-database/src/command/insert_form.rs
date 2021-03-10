use crate::model::form::Form;

use anyhow::{Context, Result};

pub async fn insert_form<'a, E>(conn: E, form: Form) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let Form {
        id,
        created_at,
        author_id,
        name,
        description,
        starts_at,
        ends_at,
        items,
    } = form;

    sqlx::query!(
        r#"
INSERT INTO forms (
    id,
    created_at,
    author_id,
    name,
    description,
    starts_at,
    ends_at,
    items
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8 )
"#,
        id,
        created_at,
        author_id,
        name,
        description,
        starts_at,
        ends_at,
        items,
    )
    .execute(conn)
    .await
    .context("Failed to insert to forms")?;

    Ok(())
}
