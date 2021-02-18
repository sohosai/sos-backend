use crate::model::string::LengthBoundedString;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormName(LengthBoundedString<typenum::U1, typenum::U64, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid form name")]
pub struct NameError {
    _priv: (),
}

impl FormName {
    pub fn from_string(name: impl Into<String>) -> Result<Self, NameError> {
        let inner = LengthBoundedString::new(name.into()).map_err(|_| NameError { _priv: () })?;
        Ok(FormName(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
