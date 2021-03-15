use crate::model::file::File;

use anyhow::{Context, Result};

pub async fn insert_file<'a, E>(conn: E, file: File) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let File {
        id,
        created_at,
        author_id,
        object_id,
        blake3_digest,
        name,
        type_,
        size,
    } = file;

    sqlx::query!(
        r#"
INSERT INTO files (
    id,
    created_at,
    author_id,
    object_id,
    blake3_digest,
    name,
    type_,
    size
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8 )
"#,
        id,
        created_at,
        author_id,
        object_id,
        blake3_digest,
        name,
        type_,
        size
    )
    .execute(conn)
    .await
    .context("Failed to insert to files")?;

    Ok(())
}
