use crate::model::string::{self, LengthBoundedString, StrippedString};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormItemName(StrippedString<LengthBoundedString<typenum::U1, typenum::U64, String>>);

#[derive(Debug, Error, Clone)]
#[error("invalid form item name")]
pub struct ItemNameError {
    _priv: (),
}

impl ItemNameError {
    fn from_length_error(_err: string::BoundedLengthError<typenum::U1, typenum::U64>) -> Self {
        ItemNameError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        ItemNameError { _priv: () }
    }
}

impl FormItemName {
    pub fn from_string(name: impl Into<String>) -> Result<Self, ItemNameError> {
        let inner =
            LengthBoundedString::new(name.into()).map_err(ItemNameError::from_length_error)?;
        let inner = StrippedString::new(inner).map_err(ItemNameError::from_not_stripped_error)?;
        Ok(FormItemName(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_inner()
    }
}
