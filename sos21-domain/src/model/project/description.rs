use crate::model::string::{self, LengthBoundedString, StrippedString};

use thiserror::Error;

/// A description text of projects, whose length is 1 ..= 50 chars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDescription(
    StrippedString<LengthBoundedString<typenum::U1, typenum::U50, String>>,
);

#[derive(Debug, Error, Clone)]
#[error("invalid project description")]
pub struct DescriptionError {
    _priv: (),
}

impl DescriptionError {
    fn from_length_error(_err: string::BoundedLengthError<typenum::U1, typenum::U50>) -> Self {
        DescriptionError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        DescriptionError { _priv: () }
    }
}

impl ProjectDescription {
    pub fn from_string(description: impl Into<String>) -> Result<Self, DescriptionError> {
        let inner = LengthBoundedString::new(description.into())
            .map_err(DescriptionError::from_length_error)?;
        let inner =
            StrippedString::new(inner).map_err(DescriptionError::from_not_stripped_error)?;
        Ok(ProjectDescription(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_inner()
    }
}
