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
    form_answer::FormAnswerId,
    project::ProjectId,
    user::UserId,
};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct FileSharingDatabase(Mutex<Transaction<'static, Postgres>>);

#[async_trait::async_trait]
impl FileSharingRepository for FileSharingDatabase {
    async fn store_file_sharing(&self, sharing: FileSharing) -> Result<()> {
        let mut lock = self.0.lock().await;

        let sharing = from_file_sharing(sharing);
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
                form_answer_id: sharing.form_answer_id,
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
}

fn from_file_sharing(sharing: FileSharing) -> data::file_sharing::FileSharing {
    let sharing = sharing.into_content();
    data::file_sharing::FileSharing {
        id: sharing.id.to_uuid(),
        created_at: sharing.created_at.utc(),
        file_id: sharing.file_id.to_uuid(),
        is_revoked: sharing.is_revoked,
        expires_at: sharing.expires_at.map(|expires_at| expires_at.utc()),
        scope: from_file_sharing_scope(sharing.scope),
        project_id: sharing
            .scope
            .project()
            .map(|project_id| project_id.to_uuid()),
        form_answer_id: sharing
            .scope
            .form_answer()
            .map(|project_id| project_id.to_uuid()),
    }
}

fn from_file_sharing_scope(scope: FileSharingScope) -> data::file_sharing::FileSharingScope {
    match scope {
        FileSharingScope::Project(_) => data::file_sharing::FileSharingScope::Project,
        FileSharingScope::FormAnswer(_) => data::file_sharing::FileSharingScope::FormAnswer,
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
        scope: to_file_sharing_scope(sharing.scope, sharing.project_id, sharing.form_answer_id)?,
    }))
}

fn to_file_sharing_scope(
    scope: data::file_sharing::FileSharingScope,
    project_id: Option<Uuid>,
    form_answer_id: Option<Uuid>,
) -> Result<FileSharingScope> {
    match scope {
        data::file_sharing::FileSharingScope::Project => {
            let project_id = project_id.context("scope = 'project' but project_id is null")?;
            Ok(FileSharingScope::Project(ProjectId::from_uuid(project_id)))
        }
        data::file_sharing::FileSharingScope::FormAnswer => {
            let form_answer_id =
                form_answer_id.context("scope = 'form_answer' but form_answer_id is null")?;
            Ok(FileSharingScope::FormAnswer(FormAnswerId::from_uuid(
                form_answer_id,
            )))
        }
        data::file_sharing::FileSharingScope::Committee => Ok(FileSharingScope::Committee),
        data::file_sharing::FileSharingScope::CommitteeOperator => {
            Ok(FileSharingScope::CommitteeOperator)
        }
        data::file_sharing::FileSharingScope::Public => Ok(FileSharingScope::Public),
    }
}
