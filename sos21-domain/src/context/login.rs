use crate::context::{
    authentication::Authentication, FileRepository, FormAnswerRepository, FormRepository,
    ObjectRepository, ProjectRepository, UserRepository,
};
use crate::model::user::User;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Login<C> {
    inner: C,
    user: User,
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("the user is not signed up")]
    NotSignedUp,
    #[error(transparent)]
    Internal(anyhow::Error),
}

impl<C> Login<C> {
    pub async fn new(inner: Authentication<C>) -> Result<Self, LoginError>
    where
        Authentication<C>: UserRepository,
    {
        let user = inner
            .get_user(inner.authenticated_user())
            .await
            .map_err(LoginError::Internal)?;

        if let Some(user) = user {
            let inner = inner.into_inner();
            Ok(Login { inner, user })
        } else {
            Err(LoginError::NotSignedUp)
        }
    }

    pub fn login_user(&self) -> &User {
        &self.user
    }

    pub fn into_inner(self) -> C {
        self.inner
    }
}

crate::delegate_project_repository! {
    impl<C: ProjectRepository + Send + Sync> ProjectRepository for Login<C> {
        self { &self.inner }
    }
}

crate::delegate_user_repository! {
    impl<C: UserRepository + Send + Sync> UserRepository for Login<C> {
        self { &self.inner }
    }
}

crate::delegate_form_repository! {
    impl<C: FormRepository + Send + Sync> FormRepository for Login<C> {
        self { &self.inner }
    }
}

crate::delegate_form_answer_repository! {
    impl<C: FormAnswerRepository + Send + Sync> FormAnswerRepository for Login<C> {
        self { &self.inner }
    }
}

crate::delegate_object_repository! {
    impl<C: ObjectRepository + Send + Sync> ObjectRepository for Login<C> {
        Self { C },
        self { &self.inner }
    }
}

crate::delegate_file_repository! {
    impl<C: FileRepository + Send + Sync> FileRepository for Login<C> {
        self { &self.inner }
    }
}