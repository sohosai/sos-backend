use crate::model::file_distribution::FileDistribution;

use anyhow::{Context, Result};

pub async fn insert_file_distribution<'a, E>(conn: E, distribution: FileDistribution) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let FileDistribution {
        id,
        created_at,
        author_id,
        name,
        description,
    } = distribution;

    sqlx::query!(
        r#"
INSERT INTO file_distributions (
    id,
    created_at,
    author_id,
    name,
    description
) VALUES ( $1, $2, $3, $4, $5 )
"#,
        id,
        created_at,
        author_id,
        name,
        description
    )
    .execute(conn)
    .await
    .context("Failed to insert to file distributions")?;

    Ok(())
}
