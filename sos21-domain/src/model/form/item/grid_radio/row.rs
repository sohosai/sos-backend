use crate::model::string::LengthBoundedString;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridRadioRowId(Uuid);

impl GridRadioRowId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        GridRadioRowId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridRadioRowLabel(LengthBoundedString<typenum::U1, typenum::U64, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid grid radio row label")]
pub struct LabelError {
    _priv: (),
}

impl GridRadioRowLabel {
    pub fn from_string(label: impl Into<String>) -> Result<Self, LabelError> {
        let inner = LengthBoundedString::new(label.into()).map_err(|_| LabelError { _priv: () })?;
        Ok(GridRadioRowLabel(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridRadioRow {
    pub id: GridRadioRowId,
    pub label: GridRadioRowLabel,
}
