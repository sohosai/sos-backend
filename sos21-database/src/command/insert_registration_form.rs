use crate::model::registration_form::RegistrationForm;

use anyhow::{Context, Result};

pub async fn insert_registration_form<'a, E>(
    conn: E,
    registration_form: RegistrationForm,
) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let RegistrationForm {
        id,
        created_at,
        author_id,
        name,
        description,
        items,
    } = registration_form;

    sqlx::query!(
        r#"
INSERT INTO registration_forms (
    id,
    created_at,
    author_id,
    name,
    description,
    items
) VALUES ( $1, $2, $3, $4, $5, $6 )
"#,
        id,
        created_at,
        author_id,
        name,
        description,
        items,
    )
    .execute(conn)
    .await
    .context("Failed to insert to registration forms")?;

    Ok(())
}
