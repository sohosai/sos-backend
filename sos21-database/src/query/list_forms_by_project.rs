use crate::model::form::Form;

use anyhow::Result;
use futures::stream::{BoxStream, StreamExt, TryStreamExt};
use uuid::Uuid;

pub fn list_forms_by_project<'a, E>(conn: E, project_id: Uuid) -> BoxStream<'a, Result<Form>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    // TODO: can't we move this logic to sos21-domain
    sqlx::query_as!(
        Form,
        r#"
SELECT
    forms.*
FROM forms
LEFT OUTER JOIN form_condition_includes
    ON form_condition_includes.form_id = forms.id
LEFT OUTER JOIN form_condition_excludes
    ON form_condition_excludes.form_id = forms.id
WHERE
    (form_condition_excludes.project_id IS NULL
        OR form_condition_excludes.project_id <> $1
    )
    AND (form_condition_includes.project_id = $1 OR (SELECT
            form_query_match(forms.unspecified_query, projects.attributes)
            OR CASE projects.category
                WHEN 'general' THEN form_query_match(forms.general_query, projects.attributes)
                WHEN 'stage'   THEN form_query_match(forms.stage_query, projects.attributes)
            END
        FROM projects
        WHERE projects.id = $1
    ))
GROUP BY forms.id
"#,
        project_id
    )
    .fetch(conn)
    .map_err(anyhow::Error::msg)
    .boxed()
}
