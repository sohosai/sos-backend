use crate::model::{
    pending_project::PendingProjectId,
    project::ProjectId,
    registration_form::RegistrationFormId,
    registration_form_answer::{RegistrationFormAnswer, RegistrationFormAnswerId},
};

use anyhow::Result;

#[async_trait::async_trait]
pub trait RegistrationFormAnswerRepository {
    async fn store_registration_form_answer(&self, answer: RegistrationFormAnswer) -> Result<()>;
    async fn get_registration_form_answer(
        &self,
        id: RegistrationFormAnswerId,
    ) -> Result<Option<RegistrationFormAnswer>>;
    async fn get_registration_form_answer_by_registration_form_and_project(
        &self,
        registration_form_id: RegistrationFormId,
        project_id: ProjectId,
    ) -> Result<Option<RegistrationFormAnswer>>;
    async fn get_registration_form_answer_by_registration_form_and_pending_project(
        &self,
        registration_form_id: RegistrationFormId,
        pending_project_id: PendingProjectId,
    ) -> Result<Option<RegistrationFormAnswer>>;
    async fn list_registration_form_answers(
        &self,
        registration_form_id: RegistrationFormId,
    ) -> Result<Vec<RegistrationFormAnswer>>;
}

#[macro_export]
macro_rules! delegate_registration_form_answer_repository {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? RegistrationFormAnswerRepository for $ty:ty {
        $sel:ident $target:block
    }) => {
        #[::async_trait::async_trait]
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::RegistrationFormAnswerRepository for $ty {
            async fn store_registration_form_answer(
                &$sel,
                answer: $crate::model::registration_form_answer::RegistrationFormAnswer
            ) -> ::anyhow::Result<()> {
                $target.store_registration_form_answer(answer).await
            }
            async fn get_registration_form_answer(
                &$sel,
                id: $crate::model::registration_form_answer::RegistrationFormAnswerId,
            ) -> ::anyhow::Result<
                Option<$crate::model::registration_form_answer::RegistrationFormAnswer>
            > {
                $target.get_registration_form_answer(id).await
            }
            async fn get_registration_form_answer_by_registration_form_and_project(
                &$sel,
                registration_form_id: $crate::model::registration_form::RegistrationFormId,
                project_id: $crate::model::project::ProjectId,
            ) -> ::anyhow::Result<
                Option<$crate::model::registration_form_answer::RegistrationFormAnswer>
            > {
                $target.get_registration_form_answer_by_registration_form_and_project(
                    registration_form_id,
                    project_id,
                )
                .await
            }
            async fn get_registration_form_answer_by_registration_form_and_pending_project(
                &$sel,
                registration_form_id: $crate::model::registration_form::RegistrationFormId,
                pending_project_id: $crate::model::pending_project::PendingProjectId,
            ) -> ::anyhow::Result<
                Option<$crate::model::registration_form_answer::RegistrationFormAnswer>
            > {
                $target.get_registration_form_answer_by_registration_form_and_pending_project(
                    registration_form_id,
                    pending_project_id,
                )
                .await
            }
            async fn list_registration_form_answers(
                &$sel,
                registration_form_id: $crate::model::registration_form::RegistrationFormId
            ) -> ::anyhow::Result<
                Vec<$crate::model::registration_form_answer::RegistrationFormAnswer>
            > {
                $target.list_registration_form_answers(registration_form_id).await
            }
        }
    }
}

#[async_trait::async_trait]
impl<C: RegistrationFormAnswerRepository + Sync> RegistrationFormAnswerRepository for &C {
    async fn store_registration_form_answer(&self, answer: RegistrationFormAnswer) -> Result<()> {
        <C as RegistrationFormAnswerRepository>::store_registration_form_answer(self, answer).await
    }

    async fn get_registration_form_answer(
        &self,
        id: RegistrationFormAnswerId,
    ) -> Result<Option<RegistrationFormAnswer>> {
        <C as RegistrationFormAnswerRepository>::get_registration_form_answer(self, id).await
    }

    async fn get_registration_form_answer_by_registration_form_and_project(
        &self,
        registration_form_id: RegistrationFormId,
        project_id: ProjectId,
    ) -> Result<Option<RegistrationFormAnswer>> {
        <C as RegistrationFormAnswerRepository>::get_registration_form_answer_by_registration_form_and_project(self, registration_form_id, project_id)
            .await
    }

    async fn get_registration_form_answer_by_registration_form_and_pending_project(
        &self,
        registration_form_id: RegistrationFormId,
        pending_project_id: PendingProjectId,
    ) -> Result<Option<RegistrationFormAnswer>> {
        <C as RegistrationFormAnswerRepository>::get_registration_form_answer_by_registration_form_and_pending_project(
            self,
            registration_form_id,
            pending_project_id
        ).await
    }

    async fn list_registration_form_answers(
        &self,
        registration_form_id: RegistrationFormId,
    ) -> Result<Vec<RegistrationFormAnswer>> {
        <C as RegistrationFormAnswerRepository>::list_registration_form_answers(
            self,
            registration_form_id,
        )
        .await
    }
}
