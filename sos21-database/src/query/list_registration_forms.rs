use crate::model::project::{ProjectAttributes, ProjectCategory};
use crate::model::registration_form::{
    RegistrationForm, RegistrationFormData, RegistrationFormProjectQueryConjunction,
};

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};

pub fn list_registration_forms<'a, 'b, E>(conn: E) -> BoxStream<'b, Result<RegistrationFormData>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'b,
    'a: 'b,
{
    sqlx::query!(
        r#"
SELECT
    registration_forms.*,
    array_agg(DISTINCT (
            registration_form_project_query_conjunctions.category,
            registration_form_project_query_conjunctions.attributes
        ))
        /* works because attributes column in registration_form_project_query_conjunctions table is NOT NULL */
        FILTER (WHERE registration_form_project_query_conjunctions.attributes IS NOT NULL)
        AS "query: Vec<(Option<ProjectCategory>, ProjectAttributes)>"
FROM registration_forms
LEFT OUTER JOIN registration_form_project_query_conjunctions
    ON registration_forms.id = registration_form_project_query_conjunctions.registration_form_id
GROUP BY registration_forms.id
"#
    )
    .fetch(conn)
    .map(|row| {
        let row = row.context("Failed to select from registration forms")?;
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
            .map(|(category, attributes)| RegistrationFormProjectQueryConjunction {
                category,
                attributes,
            })
            .collect();

        Ok(RegistrationFormData {
            registration_form,
            query,
        })
    })
    .boxed()
}
