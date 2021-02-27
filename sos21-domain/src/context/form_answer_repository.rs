use crate::model::{
    form::FormId,
    form_answer::{FormAnswer, FormAnswerId},
    project::ProjectId,
};

use anyhow::Result;

#[async_trait::async_trait]
pub trait FormAnswerRepository {
    async fn store_form_answer(&self, answer: FormAnswer) -> Result<()>;
    async fn get_form_answer(&self, id: FormAnswerId) -> Result<Option<FormAnswer>>;
    async fn get_form_answer_by_form_and_project(
        &self,
        form_id: FormId,
        project_id: ProjectId,
    ) -> Result<Option<FormAnswer>>;
    async fn list_form_answers(&self, form_id: FormId) -> Result<Vec<FormAnswer>>;
}

#[macro_export]
macro_rules! delegate_form_answer_repository {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? FormAnswerRepository for $ty:ty {
        $sel:ident $target:block
    }) => {
        #[::async_trait::async_trait]
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::FormAnswerRepository for $ty {
            async fn store_form_answer(
                &$sel,
                answer: $crate::model::form_answer::FormAnswer
            ) -> ::anyhow::Result<()> {
                $target.store_form_answer(answer).await
            }
            async fn get_form_answer(
                &$sel,
                id: $crate::model::form_answer::FormAnswerId,
            ) -> ::anyhow::Result<
                Option<$crate::model::form_answer::FormAnswer>
            > {
                $target.get_form_answer(id).await
            }
            async fn get_form_answer_by_form_and_project(
                &$sel,
                form_id: $crate::model::form::FormId,
                project_id: $crate::model::project::ProjectId,
            ) -> ::anyhow::Result<
                Option<$crate::model::form_answer::FormAnswer>
            > {
                $target.get_form_answer_by_form_and_project(form_id, project_id)
                    .await
            }
            async fn list_form_answers(
                &$sel,
                form_id: $crate::model::form::FormId
            ) -> ::anyhow::Result<
                Vec<$crate::model::form_answer::FormAnswer>
            > {
                $target.list_form_answers(form_id).await
            }
        }
    }
}

#[async_trait::async_trait]
impl<C: FormAnswerRepository + Sync> FormAnswerRepository for &C {
    async fn store_form_answer(&self, answer: FormAnswer) -> Result<()> {
        <C as FormAnswerRepository>::store_form_answer(self, answer).await
    }

    async fn get_form_answer(&self, id: FormAnswerId) -> Result<Option<FormAnswer>> {
        <C as FormAnswerRepository>::get_form_answer(self, id).await
    }

    async fn get_form_answer_by_form_and_project(
        &self,
        form_id: FormId,
        project_id: ProjectId,
    ) -> Result<Option<FormAnswer>> {
        <C as FormAnswerRepository>::get_form_answer_by_form_and_project(self, form_id, project_id)
            .await
    }

    async fn list_form_answers(&self, form_id: FormId) -> Result<Vec<FormAnswer>> {
        <C as FormAnswerRepository>::list_form_answers(self, form_id).await
    }
}
