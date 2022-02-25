use crate::model::form::{Form, FormData, FormProjectQueryConjunction};
use crate::model::project::{ProjectAttributes, ProjectCategory};

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};

pub fn list_forms<'a, E>(conn: E) -> BoxStream<'a, Result<FormData>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    sqlx::query!(
        r#"
SELECT
    forms.*,
    array_agg(DISTINCT form_condition_includes.project_id)
        FILTER (WHERE form_condition_includes.project_id IS NOT NULL)
        AS include_ids,
    array_agg(DISTINCT form_condition_excludes.project_id)
        FILTER (WHERE form_condition_excludes.project_id IS NOT NULL)
        AS exclude_ids,
    array_agg(DISTINCT (
            form_project_query_conjunctions.category,
            form_project_query_conjunctions.attributes
        ))
        /* works because attributes column in form_project_query_conjunctions table is NOT NULL */
        FILTER (WHERE form_project_query_conjunctions.attributes IS NOT NULL)
        AS "query: Vec<(Option<ProjectCategory>, ProjectAttributes)>"
FROM forms
LEFT OUTER JOIN form_condition_includes
    ON forms.id = form_condition_includes.form_id
LEFT OUTER JOIN form_condition_excludes
    ON forms.id = form_condition_excludes.form_id
LEFT OUTER JOIN form_project_query_conjunctions
    ON forms.id = form_project_query_conjunctions.form_id
GROUP BY forms.id
"#
    )
    .fetch(conn)
    .map(|row| {
        let row = row.context("Failed to select from forms")?;
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

        let include_ids = row.include_ids.unwrap_or_else(Vec::new);
        let exclude_ids = row.exclude_ids.unwrap_or_else(Vec::new);
        let query = row
            .query
            .unwrap_or_else(Vec::new)
            .into_iter()
            .map(|(category, attributes)| FormProjectQueryConjunction {
                category,
                attributes,
            })
            .collect();

        Ok(FormData {
            form,
            include_ids,
            exclude_ids,
            query,
        })
    })
    .boxed()
}
