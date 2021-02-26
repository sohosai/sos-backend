use crate::model::string::LengthBoundedString;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormAnswerItemText(LengthBoundedString<typenum::U1, typenum::U1024, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid form answer item text")]
pub struct TextError {
    _priv: (),
}

#[allow(clippy::len_without_is_empty)]
impl FormAnswerItemText {
    pub fn from_string(text: impl Into<String>) -> Result<Self, TextError> {
        let inner = LengthBoundedString::new(text.into()).map_err(|_| TextError { _priv: () })?;
        Ok(FormAnswerItemText(inner))
    }

    pub fn contains_line_break(&self) -> bool {
        let s: &str = self.0.as_ref();
        s.contains('\n')
    }

    /// it always stands that `text.len() > 0`.
    pub fn len(&self) -> usize {
        let s: &str = self.0.as_ref();
        let len = s.len();
        debug_assert!(len > 0);
        len
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
