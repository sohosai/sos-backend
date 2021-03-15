use crate::model::string::{self, LengthBoundedString};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileName(LengthBoundedString<typenum::U1, typenum::U255, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid file name")]
pub struct NameError {
    _priv: (),
}

impl NameError {
    fn from_length_error(_err: string::BoundedLengthError<typenum::U1, typenum::U255>) -> Self {
        NameError { _priv: () }
    }
}

impl FileName {
    pub fn from_string(name: impl Into<String>) -> Result<Self, NameError> {
        let inner = LengthBoundedString::new(name.into()).map_err(NameError::from_length_error)?;
        Ok(FileName(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
