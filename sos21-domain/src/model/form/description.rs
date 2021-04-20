use crate::model::bound::{Bounded, Unbounded};
use crate::model::string::{self, LengthLimitedString, StrippedString};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormDescription(
    StrippedString<LengthLimitedString<Unbounded, Bounded<typenum::U1024>, String>>,
);

#[derive(Debug, Error, Clone)]
#[error("invalid form description")]
pub struct DescriptionError {
    _priv: (),
}

impl DescriptionError {
    fn from_length_error(_err: string::LengthError<Unbounded, Bounded<typenum::U1024>>) -> Self {
        DescriptionError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        DescriptionError { _priv: () }
    }
}

impl FormDescription {
    pub fn from_string(name: impl Into<String>) -> Result<Self, DescriptionError> {
        let inner =
            LengthLimitedString::new(name.into()).map_err(DescriptionError::from_length_error)?;
        let inner =
            StrippedString::new(inner).map_err(DescriptionError::from_not_stripped_error)?;
        Ok(FormDescription(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_inner()
    }
}
