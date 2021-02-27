use crate::model::form::Form;

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_form<'a, E>(conn: E, id: Uuid) -> Result<Option<Form>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(Form, "SELECT * FROM forms WHERE id = $1", id)
        .fetch_optional(conn)
        .await
        .context("Failed to select from forms")
}
