use crate::model::file_distribution::{
    FileDistribution, FileDistributionData, FileDistributionFile,
};

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_file_distribution<'a, E>(
    conn: E,
    id: Uuid,
) -> Result<Option<FileDistributionData>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let row = sqlx::query!(
        r#"
SELECT
    file_distributions.*,
    (
        SELECT
            array_agg((
                file_distribution_files.project_id,
                file_distribution_files.sharing_id
            ))
        FROM file_distribution_files
        WHERE file_distribution_files.distribution_id = file_distributions.id
    ) AS "files: Vec<(Uuid, Uuid)>"
FROM file_distributions
WHERE file_distributions.id = $1
"#,
        id
    )
    .fetch_optional(conn)
    .await
    .context("Failed to select from file distributions")?;

    let row = match row {
        Some(row) => row,
        None => return Ok(None),
    };

    let distribution = FileDistribution {
        id: row.id,
        created_at: row.created_at,
        author_id: row.author_id,
        name: row.name,
        description: row.description,
    };

    let files = row
        .files
        .unwrap_or_else(Vec::new)
        .into_iter()
        .map(|(project_id, sharing_id)| FileDistributionFile {
            project_id,
            sharing_id,
        })
        .collect();

    Ok(Some(FileDistributionData {
        distribution,
        files,
    }))
}
