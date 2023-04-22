use crate::model::project::{ProjectAttributes, ProjectCategory};
use crate::model::registration_form::{
    RegistrationForm, RegistrationFormData, RegistrationFormProjectQueryConjunction,
};

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_registration_form<'a, E>(
    conn: E,
    id: Uuid,
) -> Result<Option<RegistrationFormData>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let row = sqlx::query!(
        r#"
SELECT
    registration_forms.*,
    (
        SELECT
            array_agg((
                registration_form_project_query_conjunctions.category,
                registration_form_project_query_conjunctions.attributes
            ))
        FROM registration_form_project_query_conjunctions
        WHERE registration_form_id = registration_forms.id
    ) AS "query: Vec<(Option<ProjectCategory>, ProjectAttributes)>"
FROM registration_forms
WHERE registration_forms.id = $1
"#,
        id
    )
    .fetch_optional(conn)
    .await
    .context("Failed to select from registration forms")?;

    let row = match row {
        Some(row) => row,
        None => return Ok(None),
    };

    let registration_form = RegistrationForm {
        id: row.id,
        created_at: row.created_at,
        author_id: row.author_id,
        name: row.name,
        description: row.description,
        items: row.items,
    };

    let query = row
        .query
        .unwrap_or_default()
        .into_iter()
        .map(
            |(category, attributes)| RegistrationFormProjectQueryConjunction {
                category,
                attributes,
            },
        )
        .collect();

    Ok(Some(RegistrationFormData {
        registration_form,
        query,
    }))
}
