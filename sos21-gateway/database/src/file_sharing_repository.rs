use crate::file_repository::to_file;

use anyhow::{Context, Result};
use futures::{future, lock::Mutex, stream::TryStreamExt};
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::FileSharingRepository;
use sos21_domain::model::{
    date_time::DateTime,
    file::{File, FileId},
    file_sharing::{FileSharing, FileSharingContent, FileSharingId, FileSharingScope},
    form::FormId,
    pending_project::PendingProjectId,
    project::ProjectId,
    registration_form::RegistrationFormId,
    registration_form_answer::RegistrationFormAnswerRespondent,
    user::UserId,
};
use sqlx::{Postgres, Transaction};

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct FileSharingDatabase(Mutex<Transaction<'static, Postgres>>);

#[async_trait::async_trait]
impl FileSharingRepository for FileSharingDatabase {
    async fn store_file_sharing(&self, sharing: FileSharing) -> Result<()> {
        let mut lock = self.0.lock().await;

        let sharing = from_file_sharing(sharing)?;
        if query::find_file_sharing(&mut *lock, sharing.id)
            .await?
            .is_some()
        {
            let input = command::update_file_sharing::Input {
                id: sharing.id,
                file_id: sharing.file_id,
                is_revoked: sharing.is_revoked,
                expires_at: sharing.expires_at,
                scope: sharing.scope,
                project_id: sharing.project_id,
                project_query: sharing.project_query,
                form_answer_project_id: sharing.form_answer_project_id,
                form_answer_form_id: sharing.form_answer_form_id,
                registration_form_answer_project_id: sharing.registration_form_answer_project_id,
                registration_form_answer_pending_project_id: sharing
                    .registration_form_answer_pending_project_id,
                registration_form_answer_registration_form_id: sharing
                    .registration_form_answer_registration_form_id,
            };
            command::update_file_sharing(&mut *lock, input).await
        } else {
            command::insert_file_sharing(&mut *lock, sharing).await
        }
    }

    async fn get_file_sharing(&self, id: FileSharingId) -> Result<Option<(FileSharing, File)>> {
        let mut lock = self.0.lock().await;
        let result = match query::find_file_sharing(&mut *lock, id.to_uuid()).await? {
            Some(x) => x,
            None => return Ok(None),
        };
        to_file_sharing_with_file(result.sharing, result.file).map(Some)
    }

    async fn list_file_sharings_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<(FileSharing, File)>> {
        let mut lock = self.0.lock().await;
        query::list_file_sharings_by_user(&mut *lock, user_id.0)
            .and_then(|result| {
                future::ready(to_file_sharing_with_file(result.sharing, result.file))
            })
            .try_collect()
            .await
    }

    async fn list_file_sharings_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<Vec<FileSharing>> {
        let mut lock = self.0.lock().await;
        query::list_file_sharings_by_pending_project(&mut *lock, pending_project_id.to_uuid())
            .and_then(|sharing| future::ready(to_file_sharing(sharing)))
            .try_collect()
            .await
    }
}

fn from_file_sharing(sharing: FileSharing) -> Result<data::file_sharing::FileSharing> {
    let sharing = sharing.into_content();
    let (form_answer_project_id, form_answer_form_id) =
        if let Some((project_id, form_id)) = sharing.scope.form_answer() {
            (Some(project_id.to_uuid()), Some(form_id.to_uuid()))
        } else {
            (None, None)
        };
    let (
        registration_form_answer_project_id,
        registration_form_answer_pending_project_id,
        registration_form_answer_registration_form_id,
    ) = if let Some((respondent, registration_form_id)) = sharing.scope.registration_form_answer() {
        let registration_form_id = registration_form_id.to_uuid();
        match respondent {
            RegistrationFormAnswerRespondent::Project(project_id) => {
                (Some(project_id.to_uuid()), None, Some(registration_form_id))
            }
            RegistrationFormAnswerRespondent::PendingProject(pending_project_id) => (
                None,
                Some(pending_project_id.to_uuid()),
                Some(registration_form_id),
            ),
        }
    } else {
        (None, None, None)
    };
    Ok(data::file_sharing::FileSharing {
        id: sharing.id.to_uuid(),
        created_at: sharing.created_at.utc(),
        file_id: sharing.file_id.to_uuid(),
        is_revoked: sharing.is_revoked,
        expires_at: sharing.expires_at.map(|expires_at| expires_at.utc()),
        scope: from_file_sharing_scope(&sharing.scope),
        project_id: sharing
            .scope
            .project()
            .map(|project_id| project_id.to_uuid()),
        project_query: sharing
            .scope
            .project_query()
            .map(serde_json::to_value)
            .transpose()?,
        form_answer_project_id,
        form_answer_form_id,
        registration_form_answer_project_id,
        registration_form_answer_pending_project_id,
        registration_form_answer_registration_form_id,
    })
}

fn from_file_sharing_scope(scope: &FileSharingScope) -> data::file_sharing::FileSharingScope {
    match scope {
        FileSharingScope::Project(_) => data::file_sharing::FileSharingScope::Project,
        FileSharingScope::ProjectQuery(_) => data::file_sharing::FileSharingScope::ProjectQuery,
        FileSharingScope::FormAnswer(_, _) => data::file_sharing::FileSharingScope::FormAnswer,
        FileSharingScope::RegistrationFormAnswer(_, _) => {
            data::file_sharing::FileSharingScope::RegistrationFormAnswer
        }
        FileSharingScope::Committee => data::file_sharing::FileSharingScope::Committee,
        FileSharingScope::CommitteeOperator => {
            data::file_sharing::FileSharingScope::CommitteeOperator
        }
        FileSharingScope::Public => data::file_sharing::FileSharingScope::Public,
    }
}

fn to_file_sharing_with_file(
    sharing: data::file_sharing::FileSharing,
    file: data::file::File,
) -> Result<(FileSharing, File)> {
    let sharing = to_file_sharing(sharing)?;
    let file = to_file(file)?;
    Ok((sharing, file))
}

fn to_file_sharing(sharing: data::file_sharing::FileSharing) -> Result<FileSharing> {
    Ok(FileSharing::from_content(FileSharingContent {
        id: FileSharingId::from_uuid(sharing.id),
        created_at: DateTime::from_utc(sharing.created_at),
        file_id: FileId::from_uuid(sharing.file_id),
        is_revoked: sharing.is_revoked,
        expires_at: sharing.expires_at.map(DateTime::from_utc),
        scope: to_file_sharing_scope(sharing)?,
    }))
}

fn to_file_sharing_scope(sharing: data::file_sharing::FileSharing) -> Result<FileSharingScope> {
    match sharing.scope {
        data::file_sharing::FileSharingScope::Project => {
            let project_id = sharing
                .project_id
                .context("scope = 'project' but project_id is null")?;
            Ok(FileSharingScope::Project(ProjectId::from_uuid(project_id)))
        }
        data::file_sharing::FileSharingScope::ProjectQuery => {
            let query = sharing
                .project_query
                .context("scope = 'project_query' but project_query is null")?;
            Ok(FileSharingScope::ProjectQuery(serde_json::from_value(
                query,
            )?))
        }
        data::file_sharing::FileSharingScope::FormAnswer => {
            let project_id = sharing
                .form_answer_project_id
                .context("scope = 'form_answer' but form_answer_project_id is null")?;
            let form_id = sharing
                .form_answer_form_id
                .context("scope = 'form_answer' but form_answer_form_id is null")?;
            Ok(FileSharingScope::FormAnswer(
                ProjectId::from_uuid(project_id),
                FormId::from_uuid(form_id),
            ))
        }
        data::file_sharing::FileSharingScope::RegistrationFormAnswer => {
            let respondent = match (sharing.registration_form_answer_project_id, sharing.registration_form_answer_pending_project_id) {
                (Some(project_id), None) => {
                    RegistrationFormAnswerRespondent::Project(ProjectId::from_uuid(project_id))
                },
                (None, Some(pending_project_id)) => {
                    RegistrationFormAnswerRespondent::PendingProject(PendingProjectId::from_uuid(pending_project_id))
                },
                (Some(_), Some(_)) => anyhow::bail!(
                    "both registration_form_answer_project_id and registration_form_answer_pending_project_id are set \
                    when scope = 'registration_form_answer'"
                ),
                (None, None) => anyhow::bail!(
                    "both registration_form_answer_project_id and registration_form_answer_pending_project_id is null \
                    when scope = 'registration_form_answer'"
                ),
            };
            let registration_form_id = sharing.registration_form_answer_registration_form_id
                .context("scope = 'registration_form_answer' but registration_form_answer_registration_form_id is null")?;
            Ok(FileSharingScope::RegistrationFormAnswer(
                respondent,
                RegistrationFormId::from_uuid(registration_form_id),
            ))
        }
        data::file_sharing::FileSharingScope::Committee => Ok(FileSharingScope::Committee),
        data::file_sharing::FileSharingScope::CommitteeOperator => {
            Ok(FileSharingScope::CommitteeOperator)
        }
        data::file_sharing::FileSharingScope::Public => Ok(FileSharingScope::Public),
    }
}
