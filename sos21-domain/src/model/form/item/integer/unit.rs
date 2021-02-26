use crate::model::string::LengthBoundedString;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IntegerFormItemUnit(LengthBoundedString<typenum::U1, typenum::U16, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid integer form item unit")]
pub struct UnitError {
    _priv: (),
}

impl IntegerFormItemUnit {
    pub fn from_string(unit: impl Into<String>) -> Result<Self, UnitError> {
        let inner = LengthBoundedString::new(unit.into()).map_err(|_| UnitError { _priv: () })?;
        Ok(IntegerFormItemUnit(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
