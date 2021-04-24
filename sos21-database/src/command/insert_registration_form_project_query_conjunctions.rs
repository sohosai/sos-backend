use crate::model::project::{ProjectAttributes, ProjectCategory};

use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProjectQueryConjunction {
    pub category: Option<ProjectCategory>,
    pub attributes: ProjectAttributes,
}

pub async fn insert_registration_form_project_query_conjunctions<'a, E>(
    conn: E,
    registration_form_id: Uuid,
    query: Vec<ProjectQueryConjunction>,
) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    // Workaround for sqlx 0.5 that cannot encode array of user-defined types
    // TODO: Encode `Vec<ProjectCategory>` directly once it is supported

    let mut categories = Vec::new();
    let mut attributes = Vec::new();

    for conj in query {
        let category = conj.category.as_ref().map(|category| match category {
            ProjectCategory::General => "general",
            ProjectCategory::Stage => "stage",
            ProjectCategory::Cooking => "cooking",
            ProjectCategory::Food => "food",
        });
        categories.push(category);
        attributes.push(conj.attributes.bits() as i32);
    }

    sqlx::query!(
        r#"
INSERT INTO registration_form_project_query_conjunctions (
    registration_form_id,
    category,
    attributes
)
SELECT
    $1 AS registration_form_id,
    query.category,
    query.attributes
FROM unnest(
    $2::project_category[],
    $3::integer[]
) AS query(
    category,
    attributes
)
"#,
        registration_form_id,
        &categories as _,
        &attributes
    )
    .execute(conn)
    .await
    .context("Failed to insert to registration form project query conjunctions")?;

    Ok(())
}
