use crate::model::string::LengthBoundedString;

use thiserror::Error;

/// A description text of projects, whose length is 1 ..= 50 chars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDescription(LengthBoundedString<typenum::U1, typenum::U50, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid project description")]
pub struct DescriptionError {
    _priv: (),
}

impl ProjectDescription {
    pub fn from_string(description: impl Into<String>) -> Result<Self, DescriptionError> {
        let inner = LengthBoundedString::new(description.into())
            .map_err(|_| DescriptionError { _priv: () })?;
        Ok(ProjectDescription(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
