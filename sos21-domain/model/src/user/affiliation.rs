use crate::string::LengthBoundedString;

use thiserror::Error;

/// A text of users' affiliation, whose length is 1 ..= 128 chars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAffiliation(LengthBoundedString<typenum::U1, typenum::U128, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid user affiliation")]
pub struct AffiliationError {
    _priv: (),
}

impl UserAffiliation {
    pub fn from_string(affiliation: impl Into<String>) -> Result<Self, AffiliationError> {
        let inner = LengthBoundedString::new(affiliation.into())
            .map_err(|_| AffiliationError { _priv: () })?;
        Ok(UserAffiliation(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
