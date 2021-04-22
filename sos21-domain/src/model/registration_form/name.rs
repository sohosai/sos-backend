use crate::model::string::{self, LengthBoundedString, StrippedString};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationFormName(
    StrippedString<LengthBoundedString<typenum::U1, typenum::U64, String>>,
);

#[derive(Debug, Error, Clone)]
#[error("invalid registration form name")]
pub struct NameError {
    _priv: (),
}

impl NameError {
    fn from_length_error(_err: string::BoundedLengthError<typenum::U1, typenum::U64>) -> Self {
        NameError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        NameError { _priv: () }
    }
}

impl RegistrationFormName {
    pub fn from_string(name: impl Into<String>) -> Result<Self, NameError> {
        let inner = LengthBoundedString::new(name.into()).map_err(NameError::from_length_error)?;
        let inner = StrippedString::new(inner).map_err(NameError::from_not_stripped_error)?;
        Ok(RegistrationFormName(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_inner()
    }
}
