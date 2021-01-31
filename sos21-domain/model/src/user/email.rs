use crate::email::{self, EmailAddress};

use thiserror::Error;

/// A valid university email address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserEmailAddress(EmailAddress);

#[derive(Debug, Clone)]
pub enum EmailAddressErrorKind {
    InvalidEmailAddress,
    NotUniversityEmailAddress,
}

#[derive(Debug, Error, Clone)]
enum EmailAddressErrorInner {
    #[error("invalid email address")]
    InvalidEmailAddress(#[source] email::FromStringError),
    #[error("not a university email address")]
    NotUniversityEmailAddress,
}

impl EmailAddressErrorInner {
    fn kind(&self) -> EmailAddressErrorKind {
        match self {
            EmailAddressErrorInner::InvalidEmailAddress(_) => {
                EmailAddressErrorKind::InvalidEmailAddress
            }
            EmailAddressErrorInner::NotUniversityEmailAddress => {
                EmailAddressErrorKind::NotUniversityEmailAddress
            }
        }
    }
}

#[derive(Debug, Error, Clone)]
#[error(transparent)]
pub struct EmailAddressError {
    inner: EmailAddressErrorInner,
}

impl EmailAddressError {
    pub fn kind(&self) -> EmailAddressErrorKind {
        self.inner.kind()
    }
}

impl UserEmailAddress {
    pub fn from_string(s: impl Into<String>) -> Result<UserEmailAddress, EmailAddressError> {
        let email = EmailAddress::from_string(s.into()).map_err(|err| EmailAddressError {
            inner: EmailAddressErrorInner::InvalidEmailAddress(err),
        })?;
        if email.ends_with("@s.tsukuba.ac.jp") || email.ends_with("@u.tsukuba.ac.jp") {
            Ok(UserEmailAddress(email))
        } else {
            Err(EmailAddressError {
                inner: EmailAddressErrorInner::NotUniversityEmailAddress,
            })
        }
    }

    pub fn into_string(self) -> String {
        self.0.into_string()
    }
}

#[cfg(test)]
mod tests {
    use super::UserEmailAddress;

    #[test]
    fn test_address_invalid() {
        assert!(UserEmailAddress::from_string("a@b.c").is_err());
        assert!(UserEmailAddress::from_string("a.b.c@de.fg").is_err());
        assert!(UserEmailAddress::from_string("ab.c@d-e.fg").is_err());
        assert!(UserEmailAddress::from_string("ab.c@coins.tsukuba.ac.jp").is_err());
    }

    #[test]
    fn test_address_valid() {
        assert!(UserEmailAddress::from_string("a@s.tsukuba.ac.jp").is_ok());
        assert!(UserEmailAddress::from_string("a.b.c@u.tsukuba.ac.jp").is_ok());
        assert!(UserEmailAddress::from_string("a-c@s.tsukuba.ac.jp").is_ok());
    }
}
