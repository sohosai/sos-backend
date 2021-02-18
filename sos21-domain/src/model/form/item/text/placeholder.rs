use crate::model::bound::{Bounded, Unbounded};
use crate::model::string::LengthLimitedString;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextFormItemPlaceholder(LengthLimitedString<Unbounded, Bounded<typenum::U1024>, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid text form item placeholder")]
pub struct PlaceholderError {
    _priv: (),
}

impl TextFormItemPlaceholder {
    pub fn from_string(name: impl Into<String>) -> Result<Self, PlaceholderError> {
        let inner =
            LengthLimitedString::new(name.into()).map_err(|_| PlaceholderError { _priv: () })?;
        Ok(TextFormItemPlaceholder(inner))
    }

    pub fn len(&self) -> usize {
        self.0.as_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.as_ref().is_empty()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
