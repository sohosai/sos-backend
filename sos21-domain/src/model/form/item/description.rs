use crate::model::bound::{Bounded, Unbounded};
use crate::model::string::LengthLimitedString;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FormItemDescription(LengthLimitedString<Unbounded, Bounded<typenum::U1024>, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid form item description")]
pub struct ItemDescriptionError {
    _priv: (),
}

impl FormItemDescription {
    pub fn from_string(name: impl Into<String>) -> Result<Self, ItemDescriptionError> {
        let inner = LengthLimitedString::new(name.into())
            .map_err(|_| ItemDescriptionError { _priv: () })?;
        Ok(FormItemDescription(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
