use crate::model::bound::{Bounded, Unbounded};
use crate::model::string::LengthLimitedString;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormDescription(LengthLimitedString<Unbounded, Bounded<typenum::U1024>, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid form description")]
pub struct DescriptionError {
    _priv: (),
}

impl FormDescription {
    pub fn from_string(name: impl Into<String>) -> Result<Self, DescriptionError> {
        let inner =
            LengthLimitedString::new(name.into()).map_err(|_| DescriptionError { _priv: () })?;
        Ok(FormDescription(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
