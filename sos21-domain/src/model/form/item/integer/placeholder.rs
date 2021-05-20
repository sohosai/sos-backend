use crate::model::bound::{Bounded, Unbounded};
use crate::model::string::LengthLimitedString;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IntegerFormItemPlaceholder(
    LengthLimitedString<Unbounded, Bounded<typenum::U1024>, String>,
);

#[derive(Debug, Error, Clone)]
#[error("invalid integer form item placeholder")]
pub struct PlaceholderError {
    _priv: (),
}

impl IntegerFormItemPlaceholder {
    pub fn from_string(name: impl Into<String>) -> Result<Self, PlaceholderError> {
        let inner =
            LengthLimitedString::new(name.into()).map_err(|_| PlaceholderError { _priv: () })?;
        Ok(IntegerFormItemPlaceholder(inner))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
