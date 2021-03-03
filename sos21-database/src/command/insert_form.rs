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
        condition,
        unspecified_query,
        general_query,
        stage_query,
        cooking_query,
        food_query,
        needs_sync,
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
    items,
    condition,
    unspecified_query,
    general_query,
    stage_query,
    cooking_query,
    food_query,
    needs_sync
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15 )
"#,
        id,
        created_at,
        author_id,
        name,
        description,
        starts_at,
        ends_at,
        items,
        condition,
        unspecified_query,
        general_query,
        stage_query,
        cooking_query,
        food_query,
        needs_sync,
    )
    .execute(conn)
    .await
    .context("Failed to insert to forms")?;

    Ok(())
}
