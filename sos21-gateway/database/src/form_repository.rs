use anyhow::Result;
use futures::{future, lock::Mutex, stream::TryStreamExt};
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::FormRepository;
use sos21_domain::model::{
    date_time::DateTime,
    form::{Form, FormCondition, FormDescription, FormId, FormName, FormPeriod},
    project::{ProjectAttribute, ProjectCategory, ProjectId},
    user::UserId,
};
use sqlx::types::BitVec;
use sqlx::{Postgres, Transaction};

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct FormDatabase(Mutex<Transaction<'static, Postgres>>);

// TODO: reduce the number of queries
#[async_trait::async_trait]
impl FormRepository for FormDatabase {
    async fn store_form(&self, form: Form) -> Result<()> {
        let mut lock = self.0.lock().await;

        if let Some(old_form) = query::find_form(&mut *lock, form.id.to_uuid()).await? {
            let condition: FormCondition = bincode::deserialize(&old_form.condition)?;

            for delete_id in form.condition.includes.difference(&condition.includes) {
                command::delete_form_condition_include(
                    &mut *lock,
                    data::form::FormConditionInclude {
                        project_id: delete_id.to_uuid(),
                        form_id: form.id.to_uuid(),
                    },
                )
                .await?;
            }
            for insert_id in condition.includes.difference(&form.condition.includes) {
                command::insert_form_condition_include(
                    &mut *lock,
                    data::form::FormConditionInclude {
                        project_id: insert_id.to_uuid(),
                        form_id: form.id.to_uuid(),
                    },
                )
                .await?;
            }

            for delete_id in form.condition.excludes.difference(&condition.excludes) {
                command::delete_form_condition_exclude(
                    &mut *lock,
                    data::form::FormConditionExclude {
                        project_id: delete_id.to_uuid(),
                        form_id: form.id.to_uuid(),
                    },
                )
                .await?;
            }
            for insert_id in condition.excludes.difference(&form.condition.excludes) {
                command::insert_form_condition_exclude(
                    &mut *lock,
                    data::form::FormConditionExclude {
                        project_id: insert_id.to_uuid(),
                        form_id: form.id.to_uuid(),
                    },
                )
                .await?;
            }

            let form = from_form(form)?;
            let input = command::update_form::Input {
                id: form.id,
                name: form.name,
                description: form.description,
                starts_at: form.starts_at,
                ends_at: form.ends_at,
                items: form.items,
                condition: form.condition,
                unspecified_query: form.unspecified_query,
                general_query: form.general_query,
                stage_query: form.stage_query,
                cooking_query: form.cooking_query,
                food_query: form.food_query,
                needs_sync: false,
            };
            command::update_form(&mut *lock, input).await
        } else {
            for include_id in form.condition.includes.projects() {
                command::insert_form_condition_include(
                    &mut *lock,
                    data::form::FormConditionInclude {
                        project_id: include_id.to_uuid(),
                        form_id: form.id.to_uuid(),
                    },
                )
                .await?;
            }
            for exclude_id in form.condition.excludes.projects() {
                command::insert_form_condition_exclude(
                    &mut *lock,
                    data::form::FormConditionExclude {
                        project_id: exclude_id.to_uuid(),
                        form_id: form.id.to_uuid(),
                    },
                )
                .await?;
            }
            command::insert_form(&mut *lock, from_form(form)?).await
        }
    }

    async fn get_form(&self, id: FormId) -> Result<Option<Form>> {
        let mut lock = self.0.lock().await;

        query::find_form(&mut *lock, id.to_uuid())
            .await
            .and_then(|opt| opt.map(to_form).transpose())
    }

    async fn list_forms(&self) -> Result<Vec<Form>> {
        let mut lock = self.0.lock().await;
        query::list_forms(&mut *lock)
            .and_then(|form| future::ready(to_form(form)))
            .try_collect()
            .await
    }

    async fn list_forms_by_project(&self, id: ProjectId) -> Result<Vec<Form>> {
        let mut lock = self.0.lock().await;
        query::list_forms_by_project(&mut *lock, id.to_uuid())
            .and_then(|form| future::ready(to_form(form)))
            .try_collect()
            .await
    }
}

fn to_form(form: data::form::Form) -> Result<Form> {
    let data::form::Form {
        id,
        created_at,
        author_id,
        name,
        description,
        starts_at,
        ends_at,
        items,
        condition,
        ..
    } = form;

    let starts_at = DateTime::from_utc(starts_at);
    let ends_at = DateTime::from_utc(ends_at);

    Ok(Form {
        id: FormId::from_uuid(id),
        created_at: DateTime::from_utc(created_at),
        author_id: UserId(author_id),
        name: FormName::from_string(name)?,
        description: FormDescription::from_string(description)?,
        period: FormPeriod::from_datetime(starts_at, ends_at)?,
        items: bincode::deserialize(&items)?,
        condition: bincode::deserialize(&condition)?,
    })
}

fn from_form(form: Form) -> Result<data::form::Form> {
    let Form {
        id,
        created_at,
        author_id,
        name,
        description,
        period,
        items,
        condition,
    } = form;

    let len = data::project::ProjectAttributes::all().bits() + 1;
    let mut unspecified_query = BitVec::from_elem(len as usize, false);
    let mut general_query = BitVec::from_elem(len as usize, false);
    let mut stage_query = BitVec::from_elem(len as usize, false);
    let mut cooking_query = BitVec::from_elem(len as usize, false);
    let mut food_query = BitVec::from_elem(len as usize, false);

    for conj in condition.query.conjunctions() {
        let query = match conj.category() {
            Some(ProjectCategory::Stage) => &mut stage_query,
            Some(ProjectCategory::General) => &mut general_query,
            Some(ProjectCategory::Cooking) => &mut cooking_query,
            Some(ProjectCategory::Food) => &mut food_query,
            None => &mut unspecified_query,
        };

        let attrs = conj
            .attributes()
            .map(|attr| match attr {
                ProjectAttribute::Academic => data::project::ProjectAttributes::ACADEMIC,
                ProjectAttribute::Artistic => data::project::ProjectAttributes::ARTISTIC,
                ProjectAttribute::Committee => data::project::ProjectAttributes::COMMITTEE,
                ProjectAttribute::Outdoor => data::project::ProjectAttributes::OUTDOOR,
            })
            .collect::<data::project::ProjectAttributes>();

        for idx in 0..len {
            let sample = data::project::ProjectAttributes::from_bits(idx).unwrap();
            if sample.contains(attrs) {
                query.set(idx as usize, true);
            }
        }
    }

    Ok(data::form::Form {
        id: id.to_uuid(),
        created_at: created_at.utc(),
        author_id: author_id.0,
        name: name.into_string(),
        description: description.into_string(),
        starts_at: period.starts_at().utc(),
        ends_at: period.ends_at().utc(),
        items: bincode::serialize(&items)?,
        condition: bincode::serialize(&condition)?,
        unspecified_query,
        general_query,
        stage_query,
        cooking_query,
        food_query,
        needs_sync: false,
    })
}
