use crate::model::string::{self, LengthBoundedString};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDistributionName(LengthBoundedString<typenum::U1, typenum::U64, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid file distribution name")]
pub struct NameError {
    _priv: (),
}

impl NameError {
    fn from_length_error(_err: string::BoundedLengthError<typenum::U1, typenum::U64>) -> Self {
        NameError { _priv: () }
    }
}

impl FileDistributionName {
    pub fn from_string(name: impl Into<String>) -> Result<Self, NameError> {
        let inner = LengthBoundedString::new(name.into()).map_err(NameError::from_length_error)?;
        Ok(FileDistributionName(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
