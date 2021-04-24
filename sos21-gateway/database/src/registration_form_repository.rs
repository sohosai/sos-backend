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
use sos21_domain::context::RegistrationFormRepository;
use sos21_domain::model::{
    date_time::DateTime,
    pending_project::PendingProjectId,
    project::ProjectId,
    project_query::{ProjectQuery, ProjectQueryConjunction},
    registration_form::{
        RegistrationForm, RegistrationFormDescription, RegistrationFormId, RegistrationFormName,
    },
    user::UserId,
};
use sqlx::{Postgres, Transaction};

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct RegistrationFormDatabase(Mutex<Transaction<'static, Postgres>>);

#[async_trait::async_trait]
impl RegistrationFormRepository for RegistrationFormDatabase {
    async fn store_registration_form(&self, registration_form: RegistrationForm) -> Result<()> {
        let mut lock = self.0.lock().await;

        let registration_form_id = registration_form.id.to_uuid();
        if query::find_registration_form(&mut *lock, registration_form_id)
            .await?
            .is_some()
        {
            let query = from_project_query(&registration_form.query);
            command::delete_registration_form_project_query_conjunctions(
                &mut *lock,
                registration_form_id,
            )
            .await?;
            command::insert_registration_form_project_query_conjunctions(
                &mut *lock,
                registration_form_id,
                query,
            )
            .await?;

            let registration_form = from_registration_form(registration_form)?;
            let input = command::update_registration_form::Input {
                id: registration_form.id,
                name: registration_form.name,
                description: registration_form.description,
                items: registration_form.items,
            };
            command::update_registration_form(&mut *lock, input).await?;
        } else {
            let query = from_project_query(&registration_form.query);
            let registration_form = from_registration_form(registration_form)?;

            command::insert_registration_form(&mut *lock, registration_form).await?;
            command::insert_registration_form_project_query_conjunctions(
                &mut *lock,
                registration_form_id,
                query,
            )
            .await?;
        }

        Ok(())
    }

    async fn get_registration_form(
        &self,
        id: RegistrationFormId,
    ) -> Result<Option<RegistrationForm>> {
        let mut lock = self.0.lock().await;

        query::find_registration_form(&mut *lock, id.to_uuid())
            .and_then(|data| future::ready(data.map(to_registration_form).transpose()))
            .await
    }

    async fn list_registration_forms(&self) -> Result<Vec<RegistrationForm>> {
        let mut lock = self.0.lock().await;

        query::list_registration_forms(&mut *lock)
            .and_then(|data| future::ready(to_registration_form(data)))
            .try_collect()
            .await
    }

    async fn list_registration_forms_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<Vec<RegistrationForm>> {
        let mut lock = self.0.lock().await;

        query::list_registration_forms_by_pending_project(&mut *lock, pending_project_id.to_uuid())
            .and_then(|data| future::ready(to_registration_form(data)))
            .try_collect()
            .await
    }

    async fn count_registration_forms_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<u64> {
        let mut lock = self.0.lock().await;

        query::count_registration_forms_by_pending_project(&mut *lock, pending_project_id.to_uuid())
            .await
    }

    async fn list_registration_forms_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<RegistrationForm>> {
        let mut lock = self.0.lock().await;

        query::list_registration_forms_by_project(&mut *lock, project_id.to_uuid())
            .and_then(|data| future::ready(to_registration_form(data)))
            .try_collect()
            .await
    }
}

fn to_registration_form(
    data: data::registration_form::RegistrationFormData,
) -> Result<RegistrationForm> {
    let data::registration_form::RegistrationForm {
        id,
        created_at,
        author_id,
        name,
        description,
        items,
    } = data.registration_form;

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

    Ok(RegistrationForm {
        id: RegistrationFormId::from_uuid(id),
        created_at: DateTime::from_utc(created_at),
        author_id: UserId(author_id),
        name: RegistrationFormName::from_string(name)?,
        description: RegistrationFormDescription::from_string(description)?,
        items: serde_json::from_value(items)?,
        query,
    })
}

fn from_registration_form(
    registration_form: RegistrationForm,
) -> Result<data::registration_form::RegistrationForm> {
    let RegistrationForm {
        id,
        created_at,
        author_id,
        name,
        description,
        items,
        query: _,
    } = registration_form;

    Ok(data::registration_form::RegistrationForm {
        id: id.to_uuid(),
        created_at: created_at.utc(),
        author_id: author_id.0,
        name: name.into_string(),
        description: description.into_string(),
        items: serde_json::to_value(&items)?,
    })
}

fn from_project_query(
    query: &ProjectQuery,
) -> Vec<command::insert_registration_form_project_query_conjunctions::ProjectQueryConjunction> {
    query
        .conjunctions()
        .map(|conj| {
            command::insert_registration_form_project_query_conjunctions::ProjectQueryConjunction {
                category: conj.category.map(from_project_category),
                attributes: from_project_attributes(&conj.attributes),
            }
        })
        .collect()
}
