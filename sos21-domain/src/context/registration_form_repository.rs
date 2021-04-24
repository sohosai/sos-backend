use crate::model::{
    pending_project::PendingProjectId,
    project::ProjectId,
    registration_form::{RegistrationForm, RegistrationFormId},
};

use anyhow::Result;

#[async_trait::async_trait]
pub trait RegistrationFormRepository {
    async fn store_registration_form(&self, registration_form: RegistrationForm) -> Result<()>;
    async fn get_registration_form(
        &self,
        id: RegistrationFormId,
    ) -> Result<Option<RegistrationForm>>;
    // TODO: Move to query service
    async fn list_registration_forms(&self) -> Result<Vec<RegistrationForm>>;
    async fn list_registration_forms_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<Vec<RegistrationForm>>;
    async fn count_registration_forms_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<u64>;
    async fn list_registration_forms_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<RegistrationForm>>;
}

#[macro_export]
macro_rules! delegate_registration_form_repository {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? RegistrationFormRepository for $ty:ty {
        $sel:ident $target:block
    }) => {
        #[::async_trait::async_trait]
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::RegistrationFormRepository for $ty {
            async fn store_registration_form(
                &$sel,
                registration_form: $crate::model::registration_form::RegistrationForm
            ) -> ::anyhow::Result<()> {
                $target.store_registration_form(registration_form).await
            }
            async fn get_registration_form(
                &$sel,
                id: $crate::model::registration_form::RegistrationFormId
            ) -> ::anyhow::Result<Option<$crate::model::registration_form::RegistrationForm>> {
                $target.get_registration_form(id).await
            }
            async fn list_registration_forms(
                &$sel
            ) -> ::anyhow::Result<Vec<$crate::model::registration_form::RegistrationForm>> {
                $target.list_registration_forms().await
            }
            async fn list_registration_forms_by_pending_project(
                &$sel,
                pending_project_id: $crate::model::pending_project::PendingProjectId
            ) -> ::anyhow::Result<Vec<$crate::model::registration_form::RegistrationForm>> {
                $target.list_registration_forms_by_pending_project(pending_project_id).await
            }
            async fn count_registration_forms_by_pending_project(
                &$sel,
                pending_project_id: $crate::model::pending_project::PendingProjectId
            ) -> ::anyhow::Result<u64> {
                $target.count_registration_forms_by_pending_project(pending_project_id).await
            }
            async fn list_registration_forms_by_project(
                &$sel,
                project_id: $crate::model::project::ProjectId
            ) -> ::anyhow::Result<Vec<$crate::model::registration_form::RegistrationForm>> {
                $target.list_registration_forms_by_project(project_id).await
            }
        }
    }
}

#[async_trait::async_trait]
impl<C: RegistrationFormRepository + Sync> RegistrationFormRepository for &C {
    async fn store_registration_form(&self, registration_form: RegistrationForm) -> Result<()> {
        <C as RegistrationFormRepository>::store_registration_form(self, registration_form).await
    }

    async fn get_registration_form(
        &self,
        id: RegistrationFormId,
    ) -> Result<Option<RegistrationForm>> {
        <C as RegistrationFormRepository>::get_registration_form(self, id).await
    }

    async fn list_registration_forms(&self) -> Result<Vec<RegistrationForm>> {
        <C as RegistrationFormRepository>::list_registration_forms(self).await
    }

    async fn list_registration_forms_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<Vec<RegistrationForm>> {
        <C as RegistrationFormRepository>::list_registration_forms_by_pending_project(
            self,
            pending_project_id,
        )
        .await
    }

    async fn count_registration_forms_by_pending_project(
        &self,
        pending_project_id: PendingProjectId,
    ) -> Result<u64> {
        <C as RegistrationFormRepository>::count_registration_forms_by_pending_project(
            self,
            pending_project_id,
        )
        .await
    }

    async fn list_registration_forms_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<RegistrationForm>> {
        <C as RegistrationFormRepository>::list_registration_forms_by_project(self, project_id)
            .await
    }
}
