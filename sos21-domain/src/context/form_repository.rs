use crate::model::{
    form::{Form, FormId},
    project::ProjectId,
};

use anyhow::Result;

#[async_trait::async_trait]
pub trait FormRepository {
    async fn store_form(&self, form: Form) -> Result<()>;
    async fn get_form(&self, id: FormId) -> Result<Option<Form>>;
    async fn list_forms(&self) -> Result<Vec<Form>>;
    async fn list_forms_by_project(&self, id: ProjectId) -> Result<Vec<Form>>;
}

#[macro_export]
macro_rules! delegate_form_repository {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? FormRepository for $ty:ty {
        $sel:ident $target:block
    }) => {
        #[::async_trait::async_trait]
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::FormRepository for $ty {
            async fn store_form(
                &$sel,
                form: $crate::model::form::Form
            ) -> ::anyhow::Result<()> {
                $target.store_form(form).await
            }
            async fn get_form(
                &$sel,
                id: $crate::model::form::FormId
            ) -> ::anyhow::Result<Option<$crate::model::form::Form>> {
                $target.get_form(id).await
            }
            async fn list_forms(
                &$sel
            ) -> ::anyhow::Result<Vec<$crate::model::form::Form>> {
                $target.list_forms().await
            }
            async fn list_forms_by_project(
                &$sel,
                id: $crate::model::project::ProjectId
            ) -> ::anyhow::Result<Vec<$crate::model::form::Form>> {
                $target.list_forms_by_project(id).await
            }
        }
    }
}

#[async_trait::async_trait]
impl<C: FormRepository + Sync> FormRepository for &C {
    async fn store_form(&self, form: Form) -> Result<()> {
        <C as FormRepository>::store_form(self, form).await
    }

    async fn get_form(&self, id: FormId) -> Result<Option<Form>> {
        <C as FormRepository>::get_form(self, id).await
    }

    async fn list_forms(&self) -> Result<Vec<Form>> {
        <C as FormRepository>::list_forms(self).await
    }

    async fn list_forms_by_project(&self, id: ProjectId) -> Result<Vec<Form>> {
        <C as FormRepository>::list_forms_by_project(self, id).await
    }
}
