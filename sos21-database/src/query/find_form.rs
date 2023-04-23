use crate::model::form::{Form, FormData, FormProjectQueryConjunction};
use crate::model::project::{ProjectAttributes, ProjectCategory};

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_form<'a, E>(conn: E, id: Uuid) -> Result<Option<FormData>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let row = sqlx::query!(
        r#"
SELECT
    forms.*,
    (
        SELECT array_agg(form_condition_includes.project_id)
        FROM form_condition_includes
        WHERE form_id = forms.id
    ) AS include_ids,
    (
        SELECT array_agg(form_condition_excludes.project_id)
        FROM form_condition_excludes
        WHERE form_id = forms.id
    ) AS exclude_ids,
    (
        SELECT
            array_agg((
                form_project_query_conjunctions.category,
                form_project_query_conjunctions.attributes
            ))
        FROM form_project_query_conjunctions
        WHERE form_id = forms.id
    ) AS "query: Vec<(Option<ProjectCategory>, ProjectAttributes)>"
FROM forms
WHERE forms.id = $1
"#,
        id
    )
    .fetch_optional(conn)
    .await
    .context("Failed to select from forms")?;

    let row = match row {
        Some(row) => row,
        None => return Ok(None),
    };

    let form = Form {
        id: row.id,
        created_at: row.created_at,
        author_id: row.author_id,
        name: row.name,
        description: row.description,
        starts_at: row.starts_at,
        ends_at: row.ends_at,
        items: row.items,
        answer_notification_webhook: row.answer_notification_webhook,
    };

    let include_ids = row.include_ids.unwrap_or_default();
    let exclude_ids = row.exclude_ids.unwrap_or_default();
    let query = row
        .query
        .unwrap_or_default()
        .into_iter()
        .map(|(category, attributes)| FormProjectQueryConjunction {
            category,
            attributes,
        })
        .collect();

    Ok(Some(FormData {
        form,
        include_ids,
        exclude_ids,
        query,
    }))
}
