use crate::model::string::{self, LengthBoundedString, StrippedString};

use thiserror::Error;

/// A text of users' affiliation, whose length is 1 ..= 128 chars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAffiliation(StrippedString<LengthBoundedString<typenum::U1, typenum::U128, String>>);

#[derive(Debug, Error, Clone)]
#[error("invalid user affiliation")]
pub struct AffiliationError {
    _priv: (),
}

impl AffiliationError {
    fn from_length_error(_err: string::BoundedLengthError<typenum::U1, typenum::U128>) -> Self {
        AffiliationError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        AffiliationError { _priv: () }
    }
}

impl UserAffiliation {
    pub fn from_string(affiliation: impl Into<String>) -> Result<Self, AffiliationError> {
        let inner = LengthBoundedString::new(affiliation.into())
            .map_err(AffiliationError::from_length_error)?;
        let inner =
            StrippedString::new(inner).map_err(AffiliationError::from_not_stripped_error)?;
        Ok(UserAffiliation(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_inner()
    }
}
