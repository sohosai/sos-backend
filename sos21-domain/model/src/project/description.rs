use crate::string::LengthBoundedString;

use thiserror::Error;

/// A description text of projects, whose length is 1 ..= 1024 chars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDescription(LengthBoundedString<typenum::U1, typenum::U1024, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid project description")]
pub struct InvalidProjectDescription {
    _priv: (),
}

impl ProjectDescription {
    pub fn from_string(description: impl Into<String>) -> Result<Self, InvalidProjectDescription> {
        let inner = LengthBoundedString::new(description.into())
            .map_err(|_| InvalidProjectDescription { _priv: () })?;
        Ok(ProjectDescription(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
