use anyhow::Result;
use futures::{future, lock::Mutex, stream::TryStreamExt};
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::FormAnswerRepository;
use sos21_domain::model::{
    date_time::DateTime,
    form::FormId,
    form_answer::{FormAnswer, FormAnswerContent, FormAnswerId},
    project::ProjectId,
    user::UserId,
};
use sqlx::{Postgres, Transaction};

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct FormAnswerDatabase(Mutex<Transaction<'static, Postgres>>);

#[async_trait::async_trait]
impl FormAnswerRepository for FormAnswerDatabase {
    async fn store_form_answer(&self, answer: FormAnswer) -> Result<()> {
        let mut lock = self.0.lock().await;

        let answer = from_form_answer(answer)?;
        if query::find_form_answer(&mut *lock, answer.id)
            .await?
            .is_some()
        {
            let input = command::update_form_answer::Input {
                id: answer.id,
                items: serde_json::to_value(&answer.items)?,
            };
            command::update_form_answer(&mut *lock, input).await
        } else {
            command::insert_form_answer(&mut *lock, answer).await
        }
    }

    async fn get_form_answer(&self, id: FormAnswerId) -> Result<Option<FormAnswer>> {
        let mut lock = self.0.lock().await;
        query::find_form_answer(&mut *lock, id.to_uuid())
            .await
            .and_then(|opt| opt.map(to_form_answer).transpose())
    }

    async fn get_form_answer_by_form_and_project(
        &self,
        form_id: FormId,
        project_id: ProjectId,
    ) -> Result<Option<FormAnswer>> {
        let mut lock = self.0.lock().await;
        query::find_form_answer_by_form_and_project(
            &mut *lock,
            form_id.to_uuid(),
            project_id.to_uuid(),
        )
        .await
        .and_then(|opt| opt.map(to_form_answer).transpose())
    }

    async fn list_form_answers(&self, form_id: FormId) -> Result<Vec<FormAnswer>> {
        let mut lock = self.0.lock().await;
        query::list_form_answers_by_form(&mut *lock, form_id.to_uuid())
            .and_then(|user| future::ready(to_form_answer(user)))
            .try_collect()
            .await
    }
}

fn to_form_answer(answer: data::form_answer::FormAnswer) -> Result<FormAnswer> {
    let data::form_answer::FormAnswer {
        id,
        created_at,
        author_id,
        form_id,
        project_id,
        items,
    } = answer;

    Ok(FormAnswer::from_content(FormAnswerContent {
        id: FormAnswerId::from_uuid(id),
        project_id: ProjectId::from_uuid(project_id),
        form_id: FormId::from_uuid(form_id),
        created_at: DateTime::from_utc(created_at),
        author_id: UserId(author_id),
        items: serde_json::from_value(items)?,
    }))
}

fn from_form_answer(answer: FormAnswer) -> Result<data::form_answer::FormAnswer> {
    let FormAnswerContent {
        id,
        project_id,
        form_id,
        created_at,
        author_id,
        items,
    } = answer.into_content();

    Ok(data::form_answer::FormAnswer {
        id: id.to_uuid(),
        created_at: created_at.utc(),
        author_id: author_id.0,
        form_id: form_id.to_uuid(),
        project_id: project_id.to_uuid(),
        items: serde_json::to_value(&items)?,
    })
}
