use crate::{ProjectRepository, UserRepository};

use sos21_domain_model::{email::EmailAddress, user::UserId};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Authentication<C> {
    inner: C,
    user_id: UserId,
    email: EmailAddress,
}

#[derive(Debug, Error, Clone)]
pub enum AuthenticationError {
    #[error("invalid email address")]
    InvalidEmail,
}

impl<C> Authentication<C> {
    pub fn new(inner: C, user_id: String, email: String) -> Result<Self, AuthenticationError> {
        let user_id = UserId(user_id);
        if let Ok(email) = EmailAddress::from_string(email) {
            Ok(Authentication {
                inner,
                user_id,
                email,
            })
        } else {
            Err(AuthenticationError::InvalidEmail)
        }
    }

    pub fn authenticated_user(&self) -> UserId {
        self.user_id.clone()
    }

    pub fn authenticated_email(&self) -> EmailAddress {
        self.email.clone()
    }

    pub fn into_inner(self) -> C {
        self.inner
    }
}

crate::delegate_project_repository! { impl<C: ProjectRepository + Send + Sync> for Authentication<C> : inner }
crate::delegate_user_repository! { impl<C: UserRepository + Send + Sync> for Authentication<C> : inner }
