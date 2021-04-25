use crate::model::file::File;
use crate::model::file_sharing::{FileSharing, FileSharingId};
use crate::model::pending_project::PendingProjectId;
use crate::model::user::UserId;

use anyhow::Result;

#[async_trait::async_trait]
pub trait FileSharingRepository {
    async fn store_file_sharing(&self, sharing: FileSharing) -> Result<()>;
    async fn get_file_sharing(&self, id: FileSharingId) -> Result<Option<(FileSharing, File)>>;
    // TODO: Move to query service
    async fn list_file_sharings_by_user(&self, user_id: UserId)
        -> Result<Vec<(FileSharing, File)>>;
    async fn list_file_sharings_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<Vec<FileSharing>>;
}

#[macro_export]
macro_rules! delegate_file_sharing_repository {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? FileSharingRepository for $ty:ty {
        $sel:ident $target:block
    }) => {
        #[::async_trait::async_trait]
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::FileSharingRepository for $ty {
            async fn store_file_sharing(
                &$sel,
                sharing: $crate::model::file_sharing::FileSharing
            ) -> ::anyhow::Result<()> {
                $target.store_file_sharing(sharing).await
            }
            async fn get_file_sharing(
                &$sel,
                id: $crate::model::file_sharing::FileSharingId
            ) -> ::anyhow::Result<Option<(
                $crate::model::file_sharing::FileSharing,
                $crate::model::file::File
            )>> {
                $target.get_file_sharing(id).await
            }
            async fn list_file_sharings_by_user(
                &$sel,
                user_id: $crate::model::user::UserId
            ) -> ::anyhow::Result<Vec<(
                $crate::model::file_sharing::FileSharing,
                $crate::model::file::File
            )>> {
                $target.list_file_sharings_by_user(user_id).await
            }
            async fn list_file_sharings_by_pending_project(
                &$sel,
                pending_project_id: $crate::model::pending_project::PendingProjectId
            ) -> ::anyhow::Result<Vec<$crate::model::file_sharing::FileSharing>> {
                $target.list_file_sharings_by_pending_project(pending_project_id).await
            }
        }
    }
}

#[async_trait::async_trait]
impl<C: FileSharingRepository + Sync> FileSharingRepository for &C {
    async fn store_file_sharing(&self, sharing: FileSharing) -> Result<()> {
        <C as FileSharingRepository>::store_file_sharing(self, sharing).await
    }

    async fn get_file_sharing(&self, id: FileSharingId) -> Result<Option<(FileSharing, File)>> {
        <C as FileSharingRepository>::get_file_sharing(self, id).await
    }

    async fn list_file_sharings_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<(FileSharing, File)>> {
        <C as FileSharingRepository>::list_file_sharings_by_user(self, user_id).await
    }

    async fn list_file_sharings_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<Vec<FileSharing>> {
        <C as FileSharingRepository>::list_file_sharings_by_pending_project(
            self,
            pending_project_id,
        )
        .await
    }
}
