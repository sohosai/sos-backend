use crate::{ProjectRepository, UserRepository};

use sos21_domain_model::user::{email, UserEmailAddress, UserId};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Authentication<C> {
    inner: C,
    user_id: UserId,
    email: UserEmailAddress,
}

#[derive(Debug, Error, Clone)]
pub enum AuthenticationError {
    #[error("invalid email address")]
    InvalidEmailAddress,
    #[error("not a university email address")]
    NotUniversityEmailAddress,
}

impl<C> Authentication<C> {
    pub fn new(inner: C, user_id: String, email: String) -> Result<Self, AuthenticationError> {
        let user_id = UserId(user_id);
        match UserEmailAddress::from_string(email) {
            Ok(email) => Ok(Authentication {
                inner,
                user_id,
                email,
            }),
            Err(err) => match err.kind() {
                email::EmailAddressErrorKind::InvalidEmailAddress => {
                    Err(AuthenticationError::InvalidEmailAddress)
                }
                email::EmailAddressErrorKind::NotUniversityEmailAddress => {
                    Err(AuthenticationError::NotUniversityEmailAddress)
                }
            },
        }
    }

    pub fn authenticated_user(&self) -> UserId {
        self.user_id.clone()
    }

    pub fn authenticated_email(&self) -> UserEmailAddress {
        self.email.clone()
    }

    pub fn into_inner(self) -> C {
        self.inner
    }
}

crate::delegate_project_repository! {
    impl<C: ProjectRepository + Send + Sync> ProjectRepository for Authentication<C> {
        self { &self.inner }
    }
}

crate::delegate_user_repository! {
    impl<C: UserRepository + Send + Sync> UserRepository for Authentication<C> {
        self { &self.inner }
    }
}
