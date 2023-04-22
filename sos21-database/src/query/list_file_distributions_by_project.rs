use crate::model::file_distribution::{
    FileDistribution, FileDistributionData, FileDistributionFile,
};

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};
use uuid::Uuid;

pub fn list_file_distributions_by_project<'a, E>(
    conn: E,
    project_id: Uuid,
) -> BoxStream<'a, Result<FileDistributionData>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    sqlx::query!(
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
LEFT OUTER JOIN file_distribution_files
    ON file_distribution_files.distribution_id = file_distributions.id
WHERE file_distribution_files.project_id = $1
GROUP BY file_distributions.id
"#,
        project_id
    )
    .fetch(conn)
    .map(|row| {
        let row = row.context("Failed to select from file distributions")?;

        let distribution = FileDistribution {
            id: row.id,
            created_at: row.created_at,
            author_id: row.author_id,
            name: row.name,
            description: row.description,
        };

        let files = row
            .files
            .unwrap_or_default()
            .into_iter()
            .map(|(project_id, sharing_id)| FileDistributionFile {
                project_id,
                sharing_id,
            })
            .collect();

        Ok(FileDistributionData {
            distribution,
            files,
        })
    })
    .boxed()
}
