use anyhow::Result;
use futures::{future, lock::Mutex, stream::TryStreamExt};
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::RegistrationFormAnswerRepository;
use sos21_domain::model::{
    date_time::DateTime,
    pending_project::PendingProjectId,
    project::ProjectId,
    registration_form::RegistrationFormId,
    registration_form_answer::{
        RegistrationFormAnswer, RegistrationFormAnswerContent, RegistrationFormAnswerId,
        RegistrationFormAnswerRespondent,
    },
    user::UserId,
};
use sqlx::{Postgres, Transaction};

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct RegistrationFormAnswerDatabase(Mutex<Transaction<'static, Postgres>>);

#[async_trait::async_trait]
impl RegistrationFormAnswerRepository for RegistrationFormAnswerDatabase {
    async fn store_registration_form_answer(&self, answer: RegistrationFormAnswer) -> Result<()> {
        let mut lock = self.0.lock().await;

        let answer = from_registration_form_answer(answer)?;
        if query::find_registration_form_answer(&mut *lock, answer.id)
            .await?
            .is_some()
        {
            let input = command::update_registration_form_answer::Input {
                id: answer.id,
                updated_at: answer.updated_at,
                project_id: answer.project_id,
                pending_project_id: answer.pending_project_id,
                items: serde_json::to_value(&answer.items)?,
            };
            command::update_registration_form_answer(&mut *lock, input).await
        } else {
            command::insert_registration_form_answer(&mut *lock, answer).await
        }
    }

    async fn get_registration_form_answer(
        &self,
        id: RegistrationFormAnswerId,
    ) -> Result<Option<RegistrationFormAnswer>> {
        let mut lock = self.0.lock().await;

        query::find_registration_form_answer(&mut *lock, id.to_uuid())
            .await
            .and_then(|opt| opt.map(to_registration_form_answer).transpose())
    }

    async fn get_registration_form_answer_by_registration_form_and_project(
        &self,
        registration_form_id: RegistrationFormId,
        project_id: ProjectId,
    ) -> Result<Option<RegistrationFormAnswer>> {
        let mut lock = self.0.lock().await;

        query::find_registration_form_answer_by_registration_form_and_project(
            &mut *lock,
            registration_form_id.to_uuid(),
            project_id.to_uuid(),
        )
        .await
        .and_then(|opt| opt.map(to_registration_form_answer).transpose())
    }

    async fn get_registration_form_answer_by_registration_form_and_pending_project(
        &self,
        registration_form_id: RegistrationFormId,
        pending_project_id: PendingProjectId,
    ) -> Result<Option<RegistrationFormAnswer>> {
        let mut lock = self.0.lock().await;

        query::find_registration_form_answer_by_registration_form_and_pending_project(
            &mut *lock,
            registration_form_id.to_uuid(),
            pending_project_id.to_uuid(),
        )
        .await
        .and_then(|opt| opt.map(to_registration_form_answer).transpose())
    }

    async fn list_registration_form_answers(
        &self,
        registration_form_id: RegistrationFormId,
    ) -> Result<Vec<RegistrationFormAnswer>> {
        let mut lock = self.0.lock().await;

        query::list_registration_form_answers_by_registration_form(
            &mut *lock,
            registration_form_id.to_uuid(),
        )
        .and_then(|answer| future::ready(to_registration_form_answer(answer)))
        .try_collect()
        .await
    }

    async fn list_registration_form_answers_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<Vec<RegistrationFormAnswer>> {
        let mut lock = self.0.lock().await;

        query::list_registration_form_answers_by_pending_project(
            &mut *lock,
            pending_project_id.to_uuid(),
        )
        .and_then(|answer| future::ready(to_registration_form_answer(answer)))
        .try_collect()
        .await
    }

    async fn count_registration_form_answers_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<u64> {
        let mut lock = self.0.lock().await;

        query::count_registration_form_answers_by_pending_project(
            &mut *lock,
            pending_project_id.to_uuid(),
        )
        .await
    }
}

fn to_registration_form_answer(
    answer: data::registration_form_answer::RegistrationFormAnswer,
) -> Result<RegistrationFormAnswer> {
    let data::registration_form_answer::RegistrationFormAnswer {
        id,
        created_at,
        updated_at,
        author_id,
        registration_form_id,
        project_id,
        pending_project_id,
        items,
    } = answer;

    let respondent = match (project_id, pending_project_id) {
        (Some(project_id), None) => {
            RegistrationFormAnswerRespondent::Project(ProjectId::from_uuid(project_id))
        }
        (None, Some(pending_project_id)) => RegistrationFormAnswerRespondent::PendingProject(
            PendingProjectId::from_uuid(pending_project_id),
        ),
        (_, _) => anyhow::bail!("invalid registration_form_answers"),
    };

    Ok(RegistrationFormAnswer::from_content(
        RegistrationFormAnswerContent {
            id: RegistrationFormAnswerId::from_uuid(id),
            created_at: DateTime::from_utc(created_at),
            updated_at: DateTime::from_utc(updated_at),
            author_id: UserId(author_id),
            registration_form_id: RegistrationFormId::from_uuid(registration_form_id),
            respondent,
            items: serde_json::from_value(items)?,
        },
    ))
}

fn from_registration_form_answer(
    answer: RegistrationFormAnswer,
) -> Result<data::registration_form_answer::RegistrationFormAnswer> {
    let RegistrationFormAnswerContent {
        id,
        created_at,
        updated_at,
        author_id,
        registration_form_id,
        respondent,
        items,
    } = answer.into_content();

    let (project_id, pending_project_id) = match respondent {
        RegistrationFormAnswerRespondent::Project(project_id) => (Some(project_id), None),
        RegistrationFormAnswerRespondent::PendingProject(pending_project_id) => {
            (None, Some(pending_project_id))
        }
    };

    Ok(data::registration_form_answer::RegistrationFormAnswer {
        id: id.to_uuid(),
        created_at: created_at.utc(),
        updated_at: updated_at.utc(),
        author_id: author_id.0,
        registration_form_id: registration_form_id.to_uuid(),
        project_id: project_id.map(|id| id.to_uuid()),
        pending_project_id: pending_project_id.map(|id| id.to_uuid()),
        items: serde_json::to_value(&items)?,
    })
}
