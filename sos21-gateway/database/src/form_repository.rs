use crate::project_repository::{
    from_project_attributes, from_project_category, to_project_attributes, to_project_category,
};

use anyhow::Result;
use futures::{
    future::{self, TryFutureExt},
    lock::Mutex,
    stream::TryStreamExt,
};
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::FormRepository;
use sos21_domain::model::{
    date_time::DateTime,
    form::{
        Form, FormCondition, FormConditionProjectSet, FormContent, FormDescription, FormId,
        FormName, FormPeriod,
    },
    project::ProjectId,
    project_query::{ProjectQuery, ProjectQueryConjunction},
    user::UserId,
};
use sqlx::{Postgres, Transaction};

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct FormDatabase(Mutex<Transaction<'static, Postgres>>);

#[async_trait::async_trait]
impl FormRepository for FormDatabase {
    async fn store_form(&self, form: Form) -> Result<()> {
        let mut lock = self.0.lock().await;

        let form_id = form.id().to_uuid();
        if let Some(old_form) = query::find_form(&mut *lock, form_id).await? {
            let old_includes = FormConditionProjectSet::from_projects(
                old_form.include_ids.into_iter().map(ProjectId::from_uuid),
            )?;
            let old_excludes = FormConditionProjectSet::from_projects(
                old_form.exclude_ids.into_iter().map(ProjectId::from_uuid),
            )?;

            command::insert_form_condition_includes(
                &mut *lock,
                form_id,
                form.condition()
                    .includes
                    .difference(&old_includes)
                    .map(|id| id.to_uuid())
                    .collect(),
            )
            .await?;
            command::delete_form_condition_includes(
                &mut *lock,
                form_id,
                old_includes
                    .difference(&form.condition().includes)
                    .map(|id| id.to_uuid())
                    .collect(),
            )
            .await?;
            command::insert_form_condition_excludes(
                &mut *lock,
                form_id,
                form.condition()
                    .excludes
                    .difference(&old_excludes)
                    .map(|id| id.to_uuid())
                    .collect(),
            )
            .await?;
            command::delete_form_condition_excludes(
                &mut *lock,
                form_id,
                old_excludes
                    .difference(&form.condition().excludes)
                    .map(|id| id.to_uuid())
                    .collect(),
            )
            .await?;

            let query = from_project_query(&form.condition().query);
            command::delete_form_project_query_conjunctions(&mut *lock, form_id).await?;
            command::insert_form_project_query_conjunctions(&mut *lock, form_id, query).await?;

            let form = from_form(form)?;
            let input = command::update_form::Input {
                id: form.id,
                name: form.name,
                description: form.description,
                starts_at: form.starts_at,
                ends_at: form.ends_at,
                items: form.items,
            };
            command::update_form(&mut *lock, input).await?;
        } else {
            let include_ids = form
                .condition()
                .includes
                .projects()
                .map(|id| id.to_uuid())
                .collect();
            let exclude_ids = form
                .condition()
                .excludes
                .projects()
                .map(|id| id.to_uuid())
                .collect();
            let query = from_project_query(&form.condition().query);
            let form = from_form(form)?;

            command::insert_form(&mut *lock, form).await?;
            command::insert_form_project_query_conjunctions(&mut *lock, form_id, query).await?;
            command::insert_form_condition_includes(&mut *lock, form_id, include_ids).await?;
            command::insert_form_condition_excludes(&mut *lock, form_id, exclude_ids).await?;
        }

        Ok(())
    }

    async fn get_form(&self, id: FormId) -> Result<Option<Form>> {
        let mut lock = self.0.lock().await;

        query::find_form(&mut *lock, id.to_uuid())
            .and_then(|data| future::ready(data.map(to_form).transpose()))
            .await
    }

    async fn list_forms(&self) -> Result<Vec<Form>> {
        let mut lock = self.0.lock().await;
        query::list_forms(&mut *lock)
            .and_then(|data| future::ready(to_form(data)))
            .try_collect()
            .await
    }

    async fn list_forms_by_project(&self, id: ProjectId) -> Result<Vec<Form>> {
        let mut lock = self.0.lock().await;
        query::list_forms_by_project(&mut *lock, id.to_uuid())
            .and_then(|data| future::ready(to_form(data)))
            .try_collect()
            .await
    }
}

fn to_form(data: data::form::FormData) -> Result<Form> {
    let data::form::Form {
        id,
        created_at,
        author_id,
        name,
        description,
        starts_at,
        ends_at,
        items,
    } = data.form;

    let starts_at = DateTime::from_utc(starts_at);
    let ends_at = DateTime::from_utc(ends_at);

    let includes = FormConditionProjectSet::from_projects(
        data.include_ids.into_iter().map(ProjectId::from_uuid),
    )?;
    let excludes = FormConditionProjectSet::from_projects(
        data.exclude_ids.into_iter().map(ProjectId::from_uuid),
    )?;
    let query = data
        .query
        .into_iter()
        .map(|conj| {
            to_project_attributes(conj.attributes).map(|attributes| ProjectQueryConjunction {
                category: conj.category.map(to_project_category),
                attributes,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let query = ProjectQuery::from_conjunctions(query)?;
    let condition = FormCondition {
        query,
        includes,
        excludes,
    };

    Ok(Form::from_content(FormContent {
        id: FormId::from_uuid(id),
        created_at: DateTime::from_utc(created_at),
        author_id: UserId(author_id),
        name: FormName::from_string(name)?,
        description: FormDescription::from_string(description)?,
        period: FormPeriod::from_datetime(starts_at, ends_at)?,
        items: serde_json::from_value(items)?,
        condition,
    }))
}

fn from_form(form: Form) -> Result<data::form::Form> {
    let FormContent {
        id,
        created_at,
        author_id,
        name,
        description,
        period,
        items,
        condition: _,
    } = form.into_content();

    Ok(data::form::Form {
        id: id.to_uuid(),
        created_at: created_at.utc(),
        author_id: author_id.0,
        name: name.into_string(),
        description: description.into_string(),
        starts_at: period.starts_at().utc(),
        ends_at: period.ends_at().utc(),
        items: serde_json::to_value(&items)?,
    })
}

fn from_project_query(
    query: &ProjectQuery,
) -> Vec<command::insert_form_project_query_conjunctions::ProjectQueryConjunction> {
    query
        .conjunctions()
        .map(
            |conj| command::insert_form_project_query_conjunctions::ProjectQueryConjunction {
                category: conj.category.map(from_project_category),
                attributes: from_project_attributes(&conj.attributes),
            },
        )
        .collect()
}
