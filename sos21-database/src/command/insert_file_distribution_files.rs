use crate::model::file_distribution::FileDistributionFile;

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn insert_file_distribution_files<'a, E>(
    conn: E,
    distribution_id: Uuid,
    files: Vec<FileDistributionFile>,
) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let mut project_ids = Vec::new();
    let mut sharing_ids = Vec::new();

    for file in files {
        project_ids.push(file.project_id);
        sharing_ids.push(file.sharing_id);
    }

    sqlx::query!(
        r#"
INSERT INTO file_distribution_files (
    distribution_id,
    project_id,
    sharing_id
)
SELECT
    $1 AS distribution_id,
    file.project_id,
    file.sharing_id
FROM unnest(
    $2::uuid[],
    $3::uuid[]
) AS file(
    project_id,
    sharing_id
)
"#,
        distribution_id,
        &project_ids,
        &sharing_ids
    )
    .execute(conn)
    .await
    .context("Failed to insert to file distribution files")?;

    Ok(())
}
