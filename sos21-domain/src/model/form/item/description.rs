use crate::model::bound::{Bounded, Unbounded};
use crate::model::string::{self, LengthLimitedString, StrippedString};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct FormItemDescription(
    StrippedString<LengthLimitedString<Unbounded, Bounded<typenum::U1024>, String>>,
);

#[derive(Debug, Error, Clone)]
#[error("invalid form item description")]
pub struct ItemDescriptionError {
    _priv: (),
}

impl ItemDescriptionError {
    fn from_length_error(_err: string::LengthError<Unbounded, Bounded<typenum::U1024>>) -> Self {
        ItemDescriptionError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        ItemDescriptionError { _priv: () }
    }
}

impl FormItemDescription {
    pub fn from_string(name: impl Into<String>) -> Result<Self, ItemDescriptionError> {
        let inner = LengthLimitedString::new(name.into())
            .map_err(ItemDescriptionError::from_length_error)?;
        let inner =
            StrippedString::new(inner).map_err(ItemDescriptionError::from_not_stripped_error)?;
        Ok(FormItemDescription(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_inner()
    }
}
