use crate::model::string::LengthBoundedString;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckboxId(Uuid);

impl CheckboxId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        CheckboxId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckboxLabel(LengthBoundedString<typenum::U1, typenum::U64, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid checkbox form item label")]
pub struct LabelError {
    _priv: (),
}

impl CheckboxLabel {
    pub fn from_string(label: impl Into<String>) -> Result<Self, LabelError> {
        let inner = LengthBoundedString::new(label.into()).map_err(|_| LabelError { _priv: () })?;
        Ok(CheckboxLabel(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkbox {
    pub id: CheckboxId,
    pub label: CheckboxLabel,
}
